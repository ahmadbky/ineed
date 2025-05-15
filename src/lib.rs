//! CLI prompting library.

#![cfg_attr(nightly, feature(doc_cfg))]
// #![warn(missing_docs, unused_allocation, missing_copy_implementations)]

use std::{
    io::{self, BufRead, Write},
    marker::PhantomData,
    ops::ControlFlow,
};

use self::format::FmtRules;

pub mod format;
pub use format::fmt;

mod promptables;
use format::rules::WrittenFmtRules;
pub use promptables::*;

pub mod prelude {
    pub use super::{Promptable as _, format::FmtRule as _};
}

pub trait Promptable {
    type Output;
    type FmtRules: FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write;

    fn prompt_with<R, W>(&mut self, mut read: R, mut write: W) -> io::Result<Self::Output>
    where
        R: BufRead,
        W: Write,
    {
        let fmt = Self::FmtRules::from(fmt());
        loop {
            if let ControlFlow::Break(out) = self.prompt_once(&mut read, &mut write, &fmt)? {
                return Ok(out);
            }
        }
    }

    fn prompt(&mut self) -> io::Result<Self::Output> {
        self.prompt_with(io::stdin().lock(), io::stdout())
    }

    fn max_tries(self, max: usize) -> MaxTries<Self>
    where
        Self: Sized,
    {
        MaxTries {
            prompt: self,
            current: 0,
            max,
        }
    }

    fn then<P, O>(self, p: P) -> Then<Self, P, O>
    where
        Self: Sized,
        P: Promptable,
        O: FromOutput<<Then<Self, P, O> as Flattenable>::RawOutput>,
    {
        Then {
            first: self,
            then: p,
            _marker: PhantomData,
        }
    }

    fn until<F>(self, until: F) -> Until<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Output) -> bool,
    {
        Until {
            prompt: self,
            until,
        }
    }

    fn map<F, T>(self, map: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> T,
    {
        Map { prompt: self, map }
    }

    fn fmt<F>(self, fmt: F) -> Formatted<Self>
    where
        Self: Sized,
        Self::FmtRules: From<F>,
    {
        Formatted {
            prompt: self,
            rules: fmt.into(),
        }
    }
}
