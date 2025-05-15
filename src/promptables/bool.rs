use std::{io, ops::ControlFlow};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

pub struct Bool<'a, 'fmt> {
    inner: WrittenInner<'a, 'fmt>,
}

pub fn bool(msg: &str) -> Bool<'_, '_> {
    Bool {
        inner: WrittenInner::new(msg),
    }
}

const TRUE_INPUTS: &[&str] = &["y", "ye", "yes", "yep", "true"];
const FALSE_INPUTS: &[&str] = &["n", "no", "nop", "nope", "nopp", "nah", "false"];

impl<'fmt> Promptable for Bool<'_, 'fmt> {
    type Output = bool;
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        let input = self.inner.prompt(read, write, fmt)?.trim().to_lowercase();
        Ok(match () {
            _ if TRUE_INPUTS.iter().any(|s| input.as_str() == *s) => ControlFlow::Break(true),
            _ if FALSE_INPUTS.iter().any(|s| input.as_str() == *s) => ControlFlow::Break(false),
            _ => ControlFlow::Continue(()),
        })
    }
}
