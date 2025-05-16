use std::{io, ops::ControlFlow};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

/// Promptable type for boolean inputs, like yes or no.
///
/// See the [`bool()`] function for more information.
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

#[cfg(test)]
mod tests {
    use std::ops::ControlFlow;

    use crate::{format::rules::WrittenFmtRules, prelude::*};

    fn test_input(input: &str, expected: bool) -> anyhow::Result<()> {
        let res = crate::bool("").prompt_with(input.as_bytes(), std::io::empty())?;
        assert_eq!(res, expected);
        Ok(())
    }

    fn test_invalid_input(input: &str) -> anyhow::Result<()> {
        let res = crate::bool("").prompt_once(
            input.as_bytes(),
            std::io::empty(),
            &WrittenFmtRules::default(),
        )?;
        assert_eq!(res, ControlFlow::Continue(()));
        Ok(())
    }

    #[test]
    fn yes_inputs() -> anyhow::Result<()> {
        for input in ["y", "ye", "yes", "Y", "YeS", "yep"] {
            test_input(input, true)?;
        }

        Ok(())
    }

    #[test]
    fn no_inputs() -> anyhow::Result<()> {
        for input in ["n", "N", "no", "nop", "nOpE"] {
            test_input(input, false)?;
        }

        Ok(())
    }

    #[test]
    fn invalid_inputs() -> anyhow::Result<()> {
        for input in ["e", "a", "s", "p", "o"]
            .into_iter()
            .flat_map(|x| [x.to_owned(), x.to_uppercase()])
        {
            test_invalid_input(input.as_str())?;
        }

        Ok(())
    }
}
