use std::{io, marker::PhantomData, ops::ControlFlow, str::FromStr};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

/// Promptable type for separated inputs of the same type.
///
/// See the [`separated()`] function for more information.
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
                .map(|s| s.trim().parse())
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn all_good_inputs() -> anyhow::Result<()> {
        let input = "1;2;3\n".as_bytes();
        let values: Vec<i32> = crate::separated("", ";").prompt_with(input, std::io::empty())?;
        assert_eq!(values, [1, 2, 3]);

        Ok(())
    }

    #[test]
    fn trim_inputs() -> anyhow::Result<()> {
        let input = " 1 ;2   ;     3\n".as_bytes();
        let values: Vec<i32> = crate::separated("", ";").prompt_with(input, std::io::empty())?;
        assert_eq!(values, [1, 2, 3]);

        Ok(())
    }

    #[test]
    fn any_invalid_input() -> anyhow::Result<()> {
        let input = "foo;2;3\n1;bar;3\n1;2;foobar\n1;2;3\n".as_bytes();
        let values: Vec<i32> = crate::separated("", ";").prompt_with(input, std::io::empty())?;
        assert_eq!(values, [1, 2, 3]);

        Ok(())
    }
}
