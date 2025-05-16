use std::{io, ops::ControlFlow};

use crate::Promptable;

/// Wrapper for promptable types to add a validator on the output.
///
/// See the [`Promptable::until()`] method for more information.
pub struct Until<P, F> {
    pub(crate) prompt: P,
    pub(crate) until: F,
}

impl<P, F> Promptable for Until<P, F>
where
    P: Promptable,
    F: FnMut(&<P as Promptable>::Output) -> bool,
{
    type Output = <P as Promptable>::Output;
    type FmtRules = <P as Promptable>::FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        self.prompt
            .prompt_once(read, write, fmt)
            .map(|flow| match flow {
                ControlFlow::Break(val) if (self.until)(&val) => ControlFlow::Break(val),
                _ => ControlFlow::Continue(()),
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn basic() -> anyhow::Result<()> {
        let input = "aa\noo\n-6\n3\n4\n10\n".as_bytes();
        let res = crate::written::<u32>("")
            .until(|x| *x > 9)
            .prompt_with(input, std::io::empty())?;
        assert_eq!(res, 10);

        Ok(())
    }
}
