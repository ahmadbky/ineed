use std::{io, marker::PhantomData, ops::ControlFlow, str::FromStr};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

pub struct Written<'a, 'fmt, T> {
    inner: WrittenInner<'a, 'fmt>,
    _marker: PhantomData<T>,
}

pub fn written<T>(msg: &str) -> Written<'_, '_, T> {
    Written {
        inner: WrittenInner::new(msg),
        _marker: PhantomData,
    }
}

impl<'fmt, T> Promptable for Written<'_, 'fmt, T>
where
    T: FromStr,
{
    type Output = T;
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        let input = self.inner.prompt(read, write, fmt)?;
        match input.parse() {
            Ok(out) if !input.is_empty() => Ok(ControlFlow::Break(out)),
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}
