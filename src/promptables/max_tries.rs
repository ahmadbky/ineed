use std::{io, ops::ControlFlow};

use crate::Promptable;

/// Wrapper for promptable types to limit the amount of tries before having a correct input.
///
/// See the [`Promptable::max_tries()`] method for more information.
pub struct MaxTries<P> {
    pub(crate) prompt: P,
    pub(crate) current: usize,
    pub(crate) max: usize,
}

/// Raised when the user exceeded the maximum amount of tries.
///
/// See [`Promptable::max_tries`] for more information.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MaxTriesExceeded(pub(crate) ());

impl<P> Promptable for MaxTries<P>
where
    P: Promptable,
{
    type Output = Result<<P as Promptable>::Output, MaxTriesExceeded>;
    type FmtRules = <P as Promptable>::FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        self.current += 1;
        if self.current > self.max {
            return Ok(ControlFlow::Break(Err(MaxTriesExceeded(()))));
        }

        self.prompt
            .prompt_once(read, write, fmt)
            .map(|flow| match flow {
                ControlFlow::Break(out) => ControlFlow::Break(Ok(out)),
                ControlFlow::Continue(_) => ControlFlow::Continue(()),
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn good_input() -> anyhow::Result<()> {
        let res = crate::written::<i32>("foo")
            .max_tries(3)
            .prompt_with("3\n".as_bytes(), std::io::empty())?;
        assert_eq!(res, Ok(3));

        Ok(())
    }

    #[test]
    fn good_input_before_max_tries() -> anyhow::Result<()> {
        let res = crate::written::<i32>("foo")
            .max_tries(3)
            .prompt_with("nop\na\n3".as_bytes(), std::io::empty())?;
        assert_eq!(res, Ok(3));

        Ok(())
    }

    #[test]
    fn max_tries_reached() -> anyhow::Result<()> {
        let res = crate::written::<i32>("foo")
            .max_tries(3)
            .prompt_with("nop\na\noo\n6".as_bytes(), std::io::empty())?;
        assert_eq!(res, Err(crate::MaxTriesExceeded(())));

        Ok(())
    }
}
