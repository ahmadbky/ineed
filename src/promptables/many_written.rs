use std::{io, marker::PhantomData, ops::ControlFlow, str::FromStr};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

/// Promptable type for many written inputs with different types.
///
/// See the [`many_written()`] function for more information.
pub struct ManyWritten<'a, 'fmt, const N: usize, O> {
    inner: WrittenInner<'a, 'fmt>,
    sep: &'a str,
    _marker: PhantomData<O>,
}

/// Returns a type that prompts the user for a determined amount of written values.
///
/// These values must be separated by the provided separator, and may have different types,
/// so the output type is a tuple that you must specify when calling the method.
///
/// There is a similar promptable: [`separated`](crate::separated). The difference is that the
/// `separated` promptable asks for any number of written values, but they must have the same type.
///
/// # Example
///
/// The below example shows how to basically use this function.
///
/// ```no_run
/// # use ineed::prelude::*;
/// let (name, age): (String, i32) = ineed::many_written("Name, age", ",").prompt().unwrap();
/// ```
///
/// Please notice how the output type (i.e. `(String, i32)`) was explictly specified.
/// This is because we need to determine at compile-time the amount of values to retrieve (the `N`
/// const generic parameter), and the types to parse into.
pub fn many_written<'a, 'fmt, O, const N: usize>(
    msg: &'a str, sep: &'a str,
) -> ManyWritten<'a, 'fmt, N, O> {
    ManyWritten {
        inner: WrittenInner::new(msg),
        sep,
        _marker: PhantomData,
    }
}

/// Used to associate a tuple of concrete types into a tuple of strings.
/// `N` is the amount of types the tuples contain.
trait TupToStrings<const N: usize> {
    type StringsTup<'a>;
}

macro_rules! impl_tup_to_strings {
    ($_Single:ident: $_single_num:literal) => {};

    ($Head:ident: $head_num:literal, $($Tail:ident: $tail_num:literal),*) => {
        impl_tup_to_strings!($($Tail: $tail_num),*);
        #[automatically_derived]
        #[diagnostic::do_not_recommend]
        impl<$Head, $($Tail),*> TupToStrings<{ $head_num $(+$tail_num)* }> for ($Head, $($Tail),*) {
            type StringsTup<'a> = (&'a str, $(<$Tail as StringType>::String<'a>),*);
        }
    };

    ($Head:ident, $($Tail:ident),*) => {
        const _: () = {
            impl_tup_to_strings!($Head:1, $($Tail:1),*);
        };
    }
}

impl_tup_to_strings! {
    A, B, C, D, E, F, G,
    H, I, J, K, L, M, N,
    O, P, Q, R, S, T, U,
    V, W, X, Y, Z
}

/// Used to parse a tuple of strings into a tuple of concrete types.
///
/// This trait is used as a bound for the output type of the [`ManyWritten`] promptable type.
#[diagnostic::on_unimplemented(
    message = "Couldn't determine the output type",
    label = "the output type must be determined from here",
    note = "try to clarify the output type of the binding, e.g. with `let x: (_, _, ...) = ...;`"
)]
trait TryFromOutput<Output> {
    fn try_from_output(output: Output) -> Option<Self>
    where
        Self: Sized;
}

/// Used for the `impl_try_from_output` macro expansion, to repeat the String type mention in tuples.
trait StringType {
    type String<'a>;
}

impl<T> StringType for T {
    type String<'a> = &'a str;
}

macro_rules! impl_try_from_output {
    (@__impl $_Single:ident) => {};

    (@__impl $Head:ident, $($Tail:ident),*) => {
        impl_try_from_output!(@__impl $($Tail),*);
        #[automatically_derived]
        #[diagnostic::do_not_recommend]
        impl<$Head, $($Tail),*> TryFromOutput<(&str, $(<$Tail as StringType>::String<'_>),*)> for ($Head, $($Tail),*)
        where
            $Head: FromStr,
            $($Tail: FromStr),*
        {
            #[allow(non_snake_case)]
            fn try_from_output(($Head, $($Tail),*): (&str, $(<$Tail as StringType>::String<'_>),*)) -> Option<Self> {
                Some((
                    $Head.parse().ok()?,
                    $($Tail.parse().ok()?),*
                ))
            }
        }
    };

    ($Head:ident, $($Tail:ident),*) => {
        const _: () = {
            impl_try_from_output!(@__impl $Head, $($Tail),*);
        };
    };
}

impl_try_from_output! {
    A, B, C, D, E, F, G,
    H, I, J, K, L, M, N,
    O, P, Q, R, S, T, U,
    V, W, X, Y, Z
}

impl<'fmt, const N: usize, O> Promptable for ManyWritten<'_, 'fmt, N, O>
where
    O: TupToStrings<N> + for<'a> TryFromOutput<<O as TupToStrings<N>>::StringsTup<'a>>,
    for<'a> <O as TupToStrings<N>>::StringsTup<'a>: From<[&'a str; N]>,
{
    type Output = O;
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: io::BufRead,
        W: io::Write,
    {
        let input = self.inner.prompt(read, write, fmt)?;
        let strings: [_; N] = match input
            .split(self.sep)
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .try_into()
        {
            Ok(array) => array,
            Err(_) => return Ok(ControlFlow::Continue(())),
        };
        match TryFromOutput::try_from_output(strings.into()) {
            Some(out) => Ok(ControlFlow::Break(out)),
            None => Ok(ControlFlow::Continue(())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn all_good_inputs() -> anyhow::Result<()> {
        let input = "foo, 1, true\n";
        let (str, i32, bool): (String, i32, bool) =
            crate::many_written("msg", ", ").prompt_with(input.as_bytes(), std::io::empty())?;

        assert_eq!(str, "foo");
        assert_eq!(i32, 1);
        assert_eq!(bool, true);

        Ok(())
    }

    #[test]
    fn trim_inputs() -> anyhow::Result<()> {
        let input = "foo, 1   ,    true\n";
        let (str, i32, bool): (String, i32, bool) =
            crate::many_written("msg", ", ").prompt_with(input.as_bytes(), std::io::empty())?;

        assert_eq!(str, "foo");
        assert_eq!(i32, 1);
        assert_eq!(bool, true);

        Ok(())
    }

    #[test]
    fn any_invalid_input() -> anyhow::Result<()> {
        let input = "foo, beg, true\nbar, 1, wow\nboor, 2, false\n";
        let (str, i32, bool): (String, i32, bool) =
            crate::many_written("msg", ", ").prompt_with(input.as_bytes(), std::io::empty())?;

        assert_eq!(str, "boor");
        assert_eq!(i32, 2);
        assert_eq!(bool, false);

        Ok(())
    }
}
