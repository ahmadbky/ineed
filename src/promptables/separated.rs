use std::{io, marker::PhantomData, ops::ControlFlow, str::FromStr};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

pub struct Separated<'a, 'fmt, I, T> {
    inner: WrittenInner<'a, 'fmt>,
    sep: &'a str,
    _marker: PhantomData<(I, T)>,
}

impl<'fmt, I, T> Promptable for Separated<'_, 'fmt, I, T>
where
    I: FromIterator<T>,
    T: FromStr,
{
    type Output = I;
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        self.inner.prompt(read, write, fmt).map(|out| {
            match out
                .split(self.sep)
                .map(str::parse)
                .collect::<Result<I, _>>()
            {
                Ok(o) if !out.is_empty() => ControlFlow::Break(o),
                _ => ControlFlow::Continue(()),
            }
        })
    }
}

pub fn separated<'a, 'fmt, I, T>(msg: &'a str, sep: &'a str) -> Separated<'a, 'fmt, I, T> {
    Separated {
        inner: WrittenInner::new(msg),
        sep,
        _marker: PhantomData,
    }
}
