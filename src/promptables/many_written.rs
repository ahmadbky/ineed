use std::{io, marker::PhantomData, ops::ControlFlow, str::FromStr};

use crate::{Promptable, WrittenFmtRules, WrittenInner};

pub struct ManyWritten<'a, 'fmt, const N: usize, O> {
    inner: WrittenInner<'a, 'fmt>,
    sep: &'a str,
    _marker: PhantomData<O>,
}

pub fn many_written<'a, 'fmt, O, const N: usize>(
    msg: &'a str, sep: &'a str,
) -> ManyWritten<'a, 'fmt, N, O> {
    ManyWritten {
        inner: WrittenInner::new(msg),
        sep,
        _marker: PhantomData,
    }
}

trait TupToStrings<const N: usize> {
    type StringsTup;
}

macro_rules! impl_tup_to_strings {
    ($_Single:ident: $_single_num:literal) => {};

    ($Head:ident: $head_num:literal, $($Tail:ident: $tail_num:literal),*) => {
        impl_tup_to_strings!($($Tail: $tail_num),*);
        impl<$Head, $($Tail),*> TupToStrings<{ $head_num $(+$tail_num)* }> for ($Head, $($Tail),*) {
            type StringsTup = (String, $(<$Tail as StringType>::String),*);
        }
    };

    ($Head:ident, $($Tail:ident),*) => {
        impl_tup_to_strings!($Head:1, $($Tail:1),*);
    }
}

impl_tup_to_strings! {
    A, B, C, D, E, F, G,
    H, I, J, K, L, M, N,
    O, P, Q, R, S, T, U,
    V, W, X, Y, Z
}

trait TryFromOutput<Output> {
    fn try_from_output(output: Output) -> Option<Self>
    where
        Self: Sized;
}

trait StringType {
    type String;
}

impl<T> StringType for T {
    type String = String;
}

macro_rules! impl_try_from_output {
    ($_Single:ident) => {};

    ($Head:ident, $($Tail:ident),*) => {
        impl_try_from_output!($($Tail),*);
        impl<$Head, $($Tail),*> TryFromOutput<(String, $(<$Tail as StringType>::String),*)> for ($Head, $($Tail),*)
        where
            $Head: FromStr,
            $($Tail: FromStr),*
        {
            #[allow(non_snake_case)]
            fn try_from_output(($Head, $($Tail),*): (String, $(<$Tail as StringType>::String),*)) -> Option<Self> {
                Some((
                    $Head.parse().ok()?,
                    $($Tail.parse().ok()?),*
                ))
            }
        }
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
    O: TupToStrings<N> + TryFromOutput<<O as TupToStrings<N>>::StringsTup>,
    <O as TupToStrings<N>>::StringsTup: From<[String; N]>,
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
            .map(str::to_owned)
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
