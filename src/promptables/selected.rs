use std::{io, marker::PhantomData, ops::ControlFlow};

use crate::{
    Promptable,
    format::{Position, Unwrappable as _, rules::SelectedFmtRules},
};

pub struct Selected<'a, 'fmt, const N: usize, T> {
    title: Option<&'a str>,
    msgs: Option<[&'a str; N]>,
    values: [Option<T>; N],
    _marker: PhantomData<&'fmt ()>,
}

impl<'fmt, const N: usize, T> Promptable for Selected<'_, 'fmt, N, T> {
    type Output = T;
    type FmtRules = SelectedFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, mut read: R, mut write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        let fmt = fmt.unwrap();
        let (open, close) = fmt.list_surrounds;

        if let Position::Top = fmt.list_msg_pos {
            if let Some(title) = if fmt.repeat_prompt {
                self.title
            } else {
                self.title.take()
            } {
                writeln!(write, "{}{}", fmt.msg_prefix, title)?;
            }
        }
        if let Some(list) = self.msgs.take() {
            for (msg, i) in list.into_iter().zip(1..) {
                writeln!(write, "{open}{i}{close} - {msg}")?;
            }
        }
        if let Position::Bottom = fmt.list_msg_pos {
            if let Some(title) = if fmt.repeat_prompt {
                self.title
            } else {
                self.title.take()
            } {
                write!(write, "{}{}", fmt.msg_prefix, title)?;
                if fmt.break_line {
                    writeln!(write)?;
                }
            }
        }

        write!(write, "{}", fmt.input_prefix)?;
        write.flush()?;

        let mut s = String::new();
        read.read_line(&mut s)?;
        let i = match s.trim().parse::<usize>() {
            Ok(i) if i >= 1 && i <= self.values.len() => i,
            _ => return Ok(ControlFlow::Continue(())),
        };

        match self.values[i - 1].take() {
            Some(out) => Ok(ControlFlow::Break(out)),
            None => Ok(ControlFlow::Continue(())),
        }
    }
}

pub fn selected<'a, 'fmt, const N: usize, T>(
    title: &'a str, list: [(&'a str, T); N],
) -> Selected<'a, 'fmt, N, T> {
    fn split<const N: usize, A, B>(arr: [(A, B); N]) -> ([A; N], [B; N]) {
        use std::array::from_fn;
        let mut arr = arr.map(|(a, b)| (Some(a), Some(b)));
        let a = from_fn(|i| arr[i].0.take().unwrap());
        let b = from_fn(|i| arr[i].1.take().unwrap());
        (a, b)
    }

    let (msgs, values) = split(list.map(|(a, b)| (a, Some(b))));

    Selected {
        title: Some(title),
        msgs: Some(msgs),
        values,
        _marker: PhantomData,
    }
}
