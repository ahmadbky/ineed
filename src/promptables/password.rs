use std::{io, ops::ControlFlow};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

/// Promptable type for passwords.
///
/// See the [`password()`] for more information.
#[cfg(feature = "rpassword")]
#[cfg_attr(nightly, doc(cfg(feature = "rpassword")))]
pub struct Password<'a, 'fmt> {
    inner: WrittenInner<'a, 'fmt>,
}

/// Returns a type that prompts a password to the user.
///
/// The password prompt uses the [`rpassword`] crate, and it ignores the input stream.
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
