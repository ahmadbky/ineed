use std::{io, ops::ControlFlow};

use crate::Promptable;

/// Wrapper for promptable types to map the output into another value.
///
/// See the [`Promptable::map()`] method for more information.
pub struct Map<P, F> {
    pub(crate) prompt: P,
    pub(crate) map: F,
}

impl<P, F, T> Promptable for Map<P, F>
where
    P: Promptable,
    F: FnMut(<P as Promptable>::Output) -> T,
{
    type Output = T;
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
                ControlFlow::Break(val) => ControlFlow::Break((self.map)(val)),
                ControlFlow::Continue(_) => ControlFlow::Continue(()),
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn basic() -> anyhow::Result<()> {
        let res = crate::written::<i32>("")
            .map(|x| x + 3)
            .prompt_with("3\n".as_bytes(), std::io::empty())?;
        assert_eq!(res, 6);

        Ok(())
    }
}
