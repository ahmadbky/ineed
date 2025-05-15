use std::{io, ops::ControlFlow};

use crate::Promptable;

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
