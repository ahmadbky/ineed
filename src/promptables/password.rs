use std::{io, ops::ControlFlow};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

#[cfg(feature = "rpassword")]
#[cfg_attr(nightly, doc(cfg(feature = "rpassword")))]
pub struct Password<'a, 'fmt> {
    inner: WrittenInner<'a, 'fmt>,
}

#[cfg(feature = "rpassword")]
#[cfg_attr(nightly, doc(cfg(feature = "rpassword")))]
pub fn password(msg: &str) -> Password<'_, '_> {
    Password {
        inner: WrittenInner::new(msg),
    }
}

#[cfg(feature = "rpassword")]
#[cfg_attr(nightly, doc(cfg(feature = "rpassword")))]
impl<'fmt> Promptable for Password<'_, 'fmt> {
    type Output = String;
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        self.inner
            .prompt_with(read, write, fmt, |_| rpassword::read_password())
            .map(|s| match s.is_empty() {
                true => ControlFlow::Continue(()),
                false => ControlFlow::Break(s),
            })
    }
}
