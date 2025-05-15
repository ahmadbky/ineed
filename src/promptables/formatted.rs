use std::{io, ops::ControlFlow};

use crate::{Promptable, format::Mergeable as _};

pub struct Formatted<P: Promptable> {
    pub(crate) prompt: P,
    pub(crate) rules: <P as Promptable>::FmtRules,
}

impl<P: Promptable> Promptable for Formatted<P> {
    type Output = <P as Promptable>::Output;
    type FmtRules = <P as Promptable>::FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        let fmt = self.rules.merge_with(fmt);
        self.prompt.prompt_once(read, write, &fmt)
    }
}
