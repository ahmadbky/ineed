use std::{io, marker::PhantomData, ops::ControlFlow};

use crate::{
    Promptable,
    format::{Expandable as _, Position, rules::SelectedFmtRules},
};

/// Promptable type for selectable inputs.
///
/// See the [`selected()`] function for more information.
pub struct Selected<'a, 'fmt, const N: usize, T> {
    title: Option<&'a str>,
    msgs: Option<[&'a str; N]>,
    values: [Option<T>; N],
    is_first_prompt: bool,
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
        let fmt = fmt.expand();
        let (open, close) = fmt.list_surrounds;

        if fmt.list_msg_pos == Position::Top && self.is_first_prompt {
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
                writeln!(write, "{open}{i}{close}{msg}")?;
            }
        }
        if fmt.list_msg_pos == Position::Bottom || !self.is_first_prompt && fmt.repeat_prompt {
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

        self.is_first_prompt = false;

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
        is_first_prompt: true,
        _marker: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::{Expandable, Position, rules::SelectedFmtRules},
        prelude::*,
    };

    #[test]
    fn normal_input() -> anyhow::Result<()> {
        let input = b"3\n".as_slice();
        let mut output = Vec::new();

        let res = crate::selected("booga", [("foo", 1000), ("bar", 2000), ("foobar", 3000)])
            .prompt_with(input, &mut output)?;
        assert_eq!(res, 3000);

        let default_fmt = SelectedFmtRules::default().expand();
        let title = format!("{}booga", default_fmt.msg_prefix);
        let expected_msg = format!(
            "{opt_title_top}\
            {open}1{close}foo\n\
            {open}2{close}bar\n\
            {open}3{close}foobar\n\
            {opt_title_bottom}{opt_nl}\
            {input_prefix}",
            opt_title_top = if let Position::Top = default_fmt.list_msg_pos {
                format!("{title}\n")
            } else {
                String::new()
            },
            open = default_fmt.list_surrounds.0,
            close = default_fmt.list_surrounds.1,
            opt_title_bottom = if let Position::Bottom = default_fmt.list_msg_pos {
                title.as_str()
            } else {
                ""
            },
            opt_nl = if default_fmt.break_line { "\n" } else { "" },
            input_prefix = default_fmt.input_prefix
        );
        assert_eq!(String::from_utf8(output)?, expected_msg);

        Ok(())
    }

    #[test]
    fn repeat_5_times_input() -> anyhow::Result<()> {
        let input = b"boo\n400\n-43\n0\n1\n".as_slice();
        let mut output = Vec::new();

        let res = crate::selected("booga", [("foo", 1000), ("bar", 2000), ("foobar", 3000)])
            .prompt_with(input, &mut output)?;
        assert_eq!(res, 1000);

        let default_fmt = SelectedFmtRules::default().expand();
        let title = format!("{}booga", default_fmt.msg_prefix);
        let expected_msg = format!(
            "{opt_title_top}\
            {open}1{close}foo\n\
            {open}2{close}bar\n\
            {open}3{close}foobar\n\
            {opt_title_bottom}{opt_nl}\
            {input_prefix}{input_prefix}{input_prefix}{input_prefix}{input_prefix}",
            opt_title_top = if let Position::Top = default_fmt.list_msg_pos {
                format!("{title}\n")
            } else {
                String::new()
            },
            open = default_fmt.list_surrounds.0,
            close = default_fmt.list_surrounds.1,
            opt_title_bottom = if let Position::Bottom = default_fmt.list_msg_pos {
                title.as_str()
            } else {
                ""
            },
            opt_nl = if default_fmt.break_line { "\n" } else { "" },
            input_prefix = default_fmt.input_prefix
        );
        assert_eq!(String::from_utf8(output)?, expected_msg);

        Ok(())
    }

    #[test]
    fn custom_fmt_top_title_without_nl_repeat_prompt() -> anyhow::Result<()> {
        let input = b"boo\nbam\nbim\n1\n".as_slice();
        let mut output = Vec::new();

        let res = crate::selected("booga", [("foo", 1000), ("bar", 2000), ("foobar", 3000)])
            .fmt(
                crate::fmt()
                    .list_msg_pos(Position::Top)
                    .break_line(false)
                    .repeat_prompt(true),
            )
            .prompt_with(input, &mut output)?;
        assert_eq!(res, 1000);

        let default_fmt = SelectedFmtRules::default().expand();
        let expected_msg = format!(
            "{msg_prefix}booga\n\
            {open}1{close}foo\n\
            {open}2{close}bar\n\
            {open}3{close}foobar\n\
            {input_prefix}\
            {msg_prefix}booga{input_prefix}\
            {msg_prefix}booga{input_prefix}\
            {msg_prefix}booga{input_prefix}",
            msg_prefix = default_fmt.msg_prefix,
            open = default_fmt.list_surrounds.0,
            close = default_fmt.list_surrounds.1,
            input_prefix = default_fmt.input_prefix
        );
        assert_eq!(String::from_utf8(output)?, expected_msg);

        Ok(())
    }

    #[test]
    fn custom_fmt_top_title_with_nl_repeat_prompt() -> anyhow::Result<()> {
        let input = b"boo\nbam\nbim\n1\n".as_slice();
        let mut output = Vec::new();

        let res = crate::selected("booga", [("foo", 1000), ("bar", 2000), ("foobar", 3000)])
            .fmt(
                crate::fmt()
                    .list_msg_pos(Position::Top)
                    .break_line(true)
                    .repeat_prompt(true),
            )
            .prompt_with(input, &mut output)?;
        assert_eq!(res, 1000);

        let default_fmt = SelectedFmtRules::default().expand();
        let expected_msg = format!(
            "{msg_prefix}booga\n\
            {open}1{close}foo\n\
            {open}2{close}bar\n\
            {open}3{close}foobar\n\
            {input_prefix}\
            {msg_prefix}booga\n{input_prefix}\
            {msg_prefix}booga\n{input_prefix}\
            {msg_prefix}booga\n{input_prefix}",
            msg_prefix = default_fmt.msg_prefix,
            open = default_fmt.list_surrounds.0,
            close = default_fmt.list_surrounds.1,
            input_prefix = default_fmt.input_prefix
        );
        assert_eq!(String::from_utf8(output)?, expected_msg);

        Ok(())
    }

    #[test]
    fn fully_customized_fmt_with_good_input() -> anyhow::Result<()> {
        let input = b"1\n".as_slice();
        let mut output = Vec::new();

        let res = crate::selected("booga", [("foo", 1000), ("bar", 2000), ("foobar", 3000)])
            .fmt(
                crate::fmt()
                    .msg_prefix("-> ")
                    .input_prefix(": ")
                    .repeat_prompt(true)
                    .break_line(false)
                    .list_surrounds("<", "> ")
                    .list_msg_pos(Position::Bottom),
            )
            .prompt_with(input, &mut output)?;
        assert_eq!(res, 1000);

        assert_eq!(
            String::from_utf8(output)?.as_str(),
            "<1> foo\n\
            <2> bar\n\
            <3> foobar\n\
            -> booga: "
        );

        Ok(())
    }

    #[test]
    fn fully_customized_fmt_with_bad_input() -> anyhow::Result<()> {
        let input = b"bim\n0\n-1\n344\n1\n".as_slice();
        let mut output = Vec::new();

        let res = crate::selected("booga", [("foo", 1000), ("bar", 2000), ("foobar", 3000)])
            .fmt(
                crate::fmt()
                    .msg_prefix("-> ")
                    .input_prefix(": ")
                    .repeat_prompt(true)
                    .break_line(false)
                    .list_surrounds("<", "> ")
                    .list_msg_pos(Position::Bottom),
            )
            .prompt_with(input, &mut output)?;
        assert_eq!(res, 1000);

        assert_eq!(
            String::from_utf8(output)?.as_str(),
            "<1> foo\n\
            <2> bar\n\
            <3> foobar\n\
            -> booga: -> booga: -> booga: -> booga: -> booga: "
        );

        Ok(())
    }
}
