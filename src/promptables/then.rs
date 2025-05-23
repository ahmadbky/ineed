use std::{io, marker::PhantomData, ops::ControlFlow};

use crate::{Promptable, format::rules::ThenFmtRules};

/// Used to convert a raw output into a proper output.
///
/// This is used as a bound for the output type of the [`Then`] promptable type, in conjunction with
/// the [`Flattenable`] trait.
#[diagnostic::on_unimplemented(
    message = "Couldn't determine the output type",
    label = "the output type must be determined from here",
    note = "try to use tuple destructuring syntax for the binding, e.g. with `let (a, b, ...) = ...`",
    note = "or clarify the output type of the binding, e.g. with `let x: {Output} = ...`"
)]
pub trait FromOutput<Output> {
    /// Converts the raw output into this type.
    fn from_output(output: Output) -> Self;
}

#[diagnostic::do_not_recommend]
impl<T> FromOutput<T> for T {
    fn from_output(output: T) -> Self {
        output
    }
}

macro_rules! impl_from_output {
    ($(
        ($($From:tt)*) into ($($Into:tt)*);
    )*) => {$(
        const _: () = {
            #[automatically_derived]
            #[diagnostic::do_not_recommend]
            impl<$($Into)*> FromOutput<($($From)*)> for ($($Into)*) {
                #[allow(non_snake_case)]
                #[inline(always)]
                fn from_output(($($From)*): ($($From)*)) -> Self {
                    ($($Into)*)
                }
            }
        };
    )*}
}

impl_from_output! {
    ((A, B), C) into (A, B, C);
    (((A, B), C), D) into (A, B, C, D);
    ((((A, B), C), D), E) into (A, B, C, D, E);
    (((((A, B), C), D), E), F) into (A, B, C, D, E, F);
    ((((((A, B), C), D), E), F), G) into (A, B, C, D, E, F, G);
    (((((((A, B), C), D), E), F), G), H) into (A, B, C, D, E, F, G, H);
    ((((((((A, B), C), D), E), F), G), H), I) into (A, B, C, D, E, F, G, H, I);
    (((((((((A, B), C), D), E), F), G), H), I), J) into (A, B, C, D, E, F, G, H, I, J);
}

/// Wrapper for chaining prompts.
///
/// See the [`Promptable::then()`] method for more information.
pub struct Then<A, B, O> {
    pub(crate) first: A,
    pub(crate) then: B,
    pub(crate) _marker: PhantomData<O>,
}

/// Represents promptable types that have an output type that is flattenable.
///
/// This is mostly used by the [`Then`] promptable type, as its raw output is nested couples
/// (e.g. `(((A, B), C), D)`), so we convert them into `(A, B, C, D)` for more convenience.
pub trait Flattenable {
    /// The raw output type (e.g. `(((A, B), C), D)`).
    type RawOutput;
}

impl<A, B, O> Flattenable for Then<A, B, O>
where
    A: Promptable,
    B: Promptable,
{
    type RawOutput = (<A as Promptable>::Output, <B as Promptable>::Output);
}

impl<A, B, O> Promptable for Then<A, B, O>
where
    A: Promptable,
    B: Promptable,
    O: FromOutput<<Self as Flattenable>::RawOutput>,
{
    type Output = O;
    type FmtRules = ThenFmtRules<<A as Promptable>::FmtRules, <B as Promptable>::FmtRules>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<O>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        prompt_twice(read, write, self, fmt)
    }
}

fn prompt_twice<R, W, A, B, O>(
    mut read: R, mut write: W, prompt: &mut Then<A, B, O>,
    fmt: &<Then<A, B, O> as Promptable>::FmtRules,
) -> io::Result<ControlFlow<O>>
where
    R: io::BufRead,
    W: io::Write,
    A: Promptable,
    B: Promptable,
    O: FromOutput<<Then<A, B, O> as Flattenable>::RawOutput>,
{
    let ControlFlow::Break(a) = prompt
        .first
        .prompt_once(&mut read, &mut write, &fmt.a_rules)?
    else {
        return Ok(ControlFlow::Continue(()));
    };

    let b = loop {
        if let ControlFlow::Break(b) =
            prompt
                .then
                .prompt_once(&mut read, &mut write, &fmt.b_rules)?
        {
            break b;
        }
    };

    Ok(ControlFlow::Break(FromOutput::from_output((a, b))))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn written_then_selected() -> anyhow::Result<()> {
        let input = "foobar\n2\nyes\n".as_bytes();
        let (foobar, int, bool) = crate::written::<String>("")
            .then(crate::selected("", [("", 1000), ("", 2000)]))
            .then(crate::bool(""))
            .prompt_with(input, std::io::empty())?;

        assert_eq!((foobar.as_str(), int, bool), ("foobar", 2000, true));

        Ok(())
    }

    #[test]
    fn any_invalid_input() -> anyhow::Result<()> {
        let input = "foobar\n1\ncaca\nfoobar\n5\nno\nfoobar\n1\nno".as_bytes();
        let (foobar, int, bool) = crate::written::<String>("")
            .then(crate::selected("", [("", 1000), ("", 2000)]))
            .then(crate::bool(""))
            .prompt_with(input, std::io::empty())?;

        assert_eq!((foobar.as_str(), int, bool), ("foobar", 1000, false));

        Ok(())
    }
}
