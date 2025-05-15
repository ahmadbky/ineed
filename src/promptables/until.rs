use std::{io, ops::ControlFlow};

use crate::Promptable;

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
