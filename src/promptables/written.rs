use std::{io, marker::PhantomData, ops::ControlFlow, str::FromStr};

use crate::{Promptable, WrittenFmtRules, format::Unwrappable as _};

pub(crate) struct WrittenInner<'a, 'fmt> {
    msg: Option<&'a str>,
    _marker: PhantomData<&'fmt ()>,
}

impl<'a> WrittenInner<'a, '_> {
    pub(crate) fn new(msg: &'a str) -> Self {
        Self {
            msg: Some(msg),
            _marker: PhantomData,
        }
    }

    pub(crate) fn prompt_with<R, W, F>(
        &mut self, mut read: R, mut write: W, fmt: &WrittenFmtRules<'_>, f: F,
    ) -> io::Result<String>
    where
        R: io::BufRead,
        W: io::Write,
        F: FnOnce(&mut R) -> io::Result<String>,
    {
        let fmt = fmt.unwrap();

        if let Some(msg) = if fmt.repeat_prompt {
            self.msg
        } else {
            self.msg.take()
        } {
            write!(write, "{}{msg}", fmt.msg_prefix)?;

            if fmt.break_line {
                writeln!(write)?;
            }
        }

        write!(write, "{}", fmt.input_prefix)?;
        write.flush()?;

        Ok(f(&mut read)?.trim().to_owned())
    }

    pub(crate) fn prompt<R, W>(
        &mut self, read: R, write: W, fmt: &WrittenFmtRules<'_>,
    ) -> io::Result<String>
    where
        R: io::BufRead,
        W: io::Write,
    {
        self.prompt_with(read, write, fmt, |read| {
            let mut s = String::new();
            read.read_line(&mut s)?;
            Ok(s)
        })
    }
}

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
