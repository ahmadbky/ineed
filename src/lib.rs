//! Lightweight CLI prompting library.
//!
//! This crate provides utility traits and types to prompt values from the user in a CLI, in a more
//! convenient way. It also allows you to customize the style of the prompts.
//!
//! For example, you can ask the user for a [written] value:
//!
//! ```no_run
//! # use ineed::prelude::*;
//! let age = ineed::written::<u8>("How old are you?").prompt().unwrap();
//! ```
//!
//! When running this instruction, it will print something similar to this:
//!
//! ```txt
//! - How old are you?
//! >
//! ```
//!
//! If the user enters an invalid input, the prompt is repeated.
//!
//! You can customize the prompt's [mod@format]:
//!
//! ```no_run
//! # use ineed::prelude::*;
//! let age = ineed::written::<u8>("How old are you?")
//!   .fmt(ineed::fmt().input_prefix(">> ").msg_prefix("-> "))
//!   .prompt()
//!   .unwrap();
//! ```
//!
//! Which will print:
//!
//! ```txt
//! -> How old are you?
//! >>
//! ```
//!
//! There are a lot of other promptable types: [selected] prompt, [password], [boolean](bool()) values,
//! etc. All the promptable types implement the [`Promptable`] trait, and support their own set of
//! custom format rules.
//!
//! There are also promptable wrappers, with the same design as Rust's iterator combination pattern.
//! For example, you can filter the output of a prompt, map the result into another value, and chain
//! it with another prompt:
//!
//! ```no_run
//! # use ineed::prelude::*;
//! enum Age {
//!   Minor,
//!   LegalAge,
//! }
//!
//! let (age, name) = ineed::written::<u8>("Your age")
//!   .until(|age| *age > 3 && *age < 120)
//!   .map(|age| match age {
//!     ..18 => Age::Minor,
//!     _ => Age::LegalAge,
//!   })
//!   .then(ineed::written::<String>("Your name"))
//!   .prompt()
//!   .unwrap();
//! ```

#![cfg_attr(nightly, feature(doc_cfg, doc_notable_trait))]
#![warn(missing_docs, unused_allocation, missing_copy_implementations)]

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

/// Exposes some traits to access their methods more conveniently.
///
/// This is intended to be used like this: `use ineed::prelude::*;`.
///
/// Promptable types aren't exposed here, so you must either import them yourself or use path syntax,
/// for example with `ineed::written(...)`.
pub mod prelude {
    pub use super::{Promptable as _, format::FmtRule as _};
}

/// Represents types that can be prompted to the console.
#[cfg_attr(nightly, doc(notable_trait))]
pub trait Promptable {
    /// The type of the output of the prompt.
    type Output;
    /// The type of styling rules the promptable supports.
    type FmtRules: FmtRules;

    /// Prompts the user for an input.
    ///
    /// It returns [`ControlFlow::Break`] with the final value if the input is correct, or
    /// [`ControlFlow::Continue`] otherwise.
    ///
    /// This method mutates the promptable, which means that the printed messages between the prompts
    /// might not be the same, and the state of the promptable itself. So you can't reuse a promptable
    /// after you used it once, and expect it to have the same behavior.
    ///
    /// This method takes some arguments for IO streams, and the prompt format.
    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write;

    /// Prompts the user for an input until it's valid.
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

    /// Prompts the user for an input until it's valid, using the standard input and output.
    fn prompt(&mut self) -> io::Result<Self::Output> {
        self.prompt_with(io::stdin().lock(), io::stdout())
    }

    /// Limits the amount of tries for the prompt to succeed.
    ///
    /// The returned value of the promptable is a result. If the user exceeds `max` tries,
    /// the returned value is `Err(`[`MaxTriesExceeded`]`)`. Otherwise, the value is wrapped
    /// inside an `Ok(...)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ineed::prelude::*;
    /// let age = ineed::written::<u8>("Your age").max_tries(3).prompt().unwrap();
    /// match age {
    ///   Ok(age) => println!("You are {age}!"),
    ///   Err(_) => println!("That's enough >:("),
    /// }
    /// ```
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

    /// Chains two prompts.
    ///
    /// The returned value is a tuple of the result of each prompt.
    ///
    /// When calling any prompt method (except [`prompt_once`][Promptable::prompt_once]), it will
    /// run all the chaining prompts at once. If any of the inputs is invalid, it will start again
    /// from the beginning of the chain.
    ///
    /// # Prompt format
    ///
    /// When giving a custom prompt format (i.e. using the [`fmt`](Promptable::fmt) method), the
    /// accepted format rules are the intersection of the accepted format rules of this promptable
    /// and the provided one.
    ///
    /// For example, if you chain a written prompt and a selected prompt, the accepted format rules
    /// are those that are accepted by both the written prompt and selected prompt
    /// (e.g. input prefix, etc).
    ///
    /// You can still specify a custom format for any promptable in the chain. If you provide
    /// the same format rule for the chain, and for a specific promptable, the rule that is
    /// used when prompting the latter is the one defined specifically for the latter, as it has
    /// more precedence over the format rules of the whole chain.
    ///
    /// # Examples
    ///
    /// The below code shows how to basically use this method:
    ///
    /// ```no_run
    /// # use ineed::prelude::*;
    /// let (x, y) = ineed::written::<i32>("X")
    ///   .then(ineed::written::<i32>("Y"))
    ///   .prompt()
    ///   .unwrap();
    /// println!("{x} + {y} = {}!", x + y);
    /// ```
    ///
    /// You can compose custom formats for the chain, and for the nested promptables:
    ///
    /// ```no_run
    /// # use ineed::prelude::*;
    /// let (x, y) = ineed::written::<i32>("X")
    ///   .then(
    ///     ineed::written::<i32>("Y")
    ///       .fmt(ineed::fmt().input_prefix(">> "))
    ///   )
    ///   .fmt(ineed::fmt().input_prefix("=> "))
    ///   .prompt()
    ///   .unwrap();
    /// ```
    ///
    /// The input prefix of the first prompt is `=> ` (defined for the whole chain), but the input
    /// prefix of the second prompt is `>> `, as it is provided for it specifically.
    ///
    /// In this example:
    ///
    /// ```no_run
    /// # use ineed::prelude::*;
    /// let (x, y) = ineed::written::<i32>("X")
    ///   .fmt(ineed::fmt().input_prefix(">> "))
    ///   .then(ineed::written::<i32>("Y"))
    ///   .fmt(ineed::fmt().input_prefix("=> "))
    ///   .prompt()
    ///   .unwrap();
    /// ```
    ///
    /// The input prefix of the first prompt is `>> `, as it is provided specifically for it,
    /// but the input prefix of the second prompt is `=> `, as it is defined for the whole chain.
    fn then<P, O>(self, promptable: P) -> Then<Self, P, O>
    where
        Self: Sized,
        P: Promptable,
        O: FromOutput<<Then<Self, P, O> as Flattenable>::RawOutput>,
    {
        Then {
            first: self,
            then: promptable,
            _marker: PhantomData,
        }
    }

    /// Adds a filter to the user input, before validating it.
    ///
    /// The given function returns whether the value entered by the user is valid or not.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ineed::prelude::*;
    /// let age = ineed::written::<u8>("You age")
    ///   .until(|age| *age > 3 && *age < 120)
    ///   .prompt()
    ///   .unwrap();
    /// ```
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

    /// Maps the user input into another value.
    ///
    /// The given function takes the value entered by the user, and returns a new value of any type
    /// from it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ineed::prelude::*;
    /// let input = ineed::written::<u8>("Your age").map(Some).prompt().unwrap();
    /// ```
    fn map<F, T>(self, map: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> T,
    {
        Map { prompt: self, map }
    }

    /// Gives the promptable a custom format.
    ///
    /// The custom format must be compatible with the promptable type. This compatibility
    /// is represented by the `From<...>` implementations for the promptable's
    /// [`FmtRules`][Promptable::FmtRules] type.
    ///
    /// See the [`format`](mod@format) module for more information.
    ///
    /// # Example
    ///
    /// The below example will print the message with the default format, except the input prefix
    /// that will be `>> `.
    ///
    /// ```no_run
    /// # use ineed::prelude::*;
    /// let name = ineed::written::<String>("Your username")
    ///   .fmt(ineed::fmt().input_prefix(">> "))
    ///   .prompt()
    ///   .unwrap();
    /// ```
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
