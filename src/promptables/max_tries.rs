use std::{io, ops::ControlFlow};

use crate::Promptable;

pub struct MaxTries<P> {
    pub(crate) prompt: P,
    pub(crate) current: usize,
    pub(crate) max: usize,
}

#[derive(thiserror::Error, Debug)]
#[error("max tries exceeded")]
pub struct MaxTriesExceeded;

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
            return Ok(ControlFlow::Break(Err(MaxTriesExceeded)));
        }

        self.prompt
            .prompt_once(read, write, fmt)
            .map(|flow| match flow {
                ControlFlow::Break(out) => ControlFlow::Break(Ok(out)),
                ControlFlow::Continue(_) => ControlFlow::Continue(()),
            })
    }
}
