use std::{
    io::{self, BufRead, Write},
    marker::PhantomData,
    ops::ControlFlow,
    str::FromStr,
};

use self::format::{FmtRule, FromDefaults, PromptFormat};

pub mod format;

pub struct MaxTries<P> {
    prompt: P,
    current: usize,
    max: usize,
}

pub struct Then<A, B, O> {
    first: A,
    then: B,
    _marker: PhantomData<O>,
}

pub struct ThenFmt<A, B, O> {
    inner: Then<A, B, O>,
}

pub trait Flattenable {
    type RawOutput;
}

impl<A, B, O> Flattenable for Then<A, B, O>
where
    A: Promptable,
    B: Promptable,
{
    type RawOutput = (<A as Promptable>::Output, <B as Promptable>::Output);
}

pub struct Until<P, F> {
    prompt: P,
    until: F,
}

pub struct Map<P, F> {
    prompt: P,
    map: F,
}

pub trait FromOutput<Output> {
    fn from_output(output: Output) -> Self;
}

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

pub struct Formatted<P>
where
    P: Promptable,
{
    prompt: P,
    fmt: PromptFormat<<P as Promptable>::FmtRule>,
}

pub trait Promptable: Sized {
    type Output;
    type FmtRule: FmtRule;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write;

    fn prompt_with<R, W>(&mut self, mut read: R, mut write: W) -> io::Result<Self::Output>
    where
        R: BufRead,
        W: Write,
    {
        let fmt = PromptFormat::default();
        loop {
            if let ControlFlow::Break(out) = self.prompt_once(&mut read, &mut write, &fmt)? {
                return Ok(out);
            }
        }
    }

    fn prompt(&mut self) -> io::Result<Self::Output> {
        self.prompt_with(io::stdin().lock(), io::stdout())
    }

    fn max_tries(self, max: usize) -> MaxTries<Self> {
        MaxTries {
            prompt: self,
            current: 0,
            max,
        }
    }

    fn then<P, O>(self, p: P) -> Then<Self, P, O>
    where
        P: Promptable,
        O: FromOutput<<Then<Self, P, O> as Flattenable>::RawOutput>,
    {
        Then {
            first: self,
            then: p,
            _marker: PhantomData,
        }
    }

    fn then_fmt<P, O>(self, p: P) -> ThenFmt<Self, P, O>
    where
        P: Promptable<FmtRule = Self::FmtRule>,
        O: FromOutput<<Then<Self, P, O> as Flattenable>::RawOutput>,
    {
        ThenFmt {
            inner: self.then(p),
        }
    }

    fn until<F>(self, until: F) -> Until<Self, F>
    where
        F: FnMut(&Self::Output) -> bool,
    {
        Until {
            prompt: self,
            until,
        }
    }

    fn map<F, T>(self, map: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> T,
    {
        Map { prompt: self, map }
    }

    fn fmt<I>(self, fmt: I) -> Formatted<Self>
    where
        I: IntoIterator<Item = Self::FmtRule>,
    {
        Formatted {
            prompt: self,
            fmt: PromptFormat {
                rules: fmt.into_iter().collect(),
            },
        }
    }
}

impl<P> Promptable for Formatted<P>
where
    P: Promptable,
{
    type Output = <P as Promptable>::Output;
    type FmtRule = <P as Promptable>::FmtRule;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let merged_fmt = self.fmt.merge_with(fmt);
        self.prompt.prompt_once(read, write, &merged_fmt)
    }
}

impl<P, F, T> Promptable for Map<P, F>
where
    P: Promptable,
    F: FnMut(<P as Promptable>::Output) -> T,
{
    type Output = T;
    type FmtRule = <P as Promptable>::FmtRule;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        self.prompt
            .prompt_once(read, write, fmt)
            .map(|flow| match flow {
                ControlFlow::Break(val) => ControlFlow::Break((self.map)(val)),
                ControlFlow::Continue(_) => ControlFlow::Continue(()),
            })
    }
}

impl<P, F> Promptable for Until<P, F>
where
    P: Promptable,
    F: FnMut(&<P as Promptable>::Output) -> bool,
{
    type Output = <P as Promptable>::Output;
    type FmtRule = <P as Promptable>::FmtRule;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        self.prompt
            .prompt_once(read, write, fmt)
            .map(|flow| match flow {
                ControlFlow::Break(val) if (self.until)(&val) => ControlFlow::Break(val),
                _ => ControlFlow::Continue(()),
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThenFmtRule<A, B> {
    First(A),
    Next(B),
}

impl<A, B> FmtRule for ThenFmtRule<A, B>
where
    A: FmtRule,
    B: FmtRule,
{
    type Defaults = Vec<Self>;

    type Struct = ThenFmtRules<<A as FmtRule>::Struct, <B as FmtRule>::Struct>;

    fn defaults() -> Self::Defaults {
        <A as FmtRule>::defaults()
            .into_iter()
            .map(ThenFmtRule::First)
            .chain(
                <B as FmtRule>::defaults()
                    .into_iter()
                    .map(ThenFmtRule::Next),
            )
            .collect()
    }

    fn fill(acc: Self::Struct, this: Self) -> Self::Struct {
        match this {
            ThenFmtRule::First(first) => ThenFmtRules {
                a_rules: <A as FmtRule>::fill(acc.a_rules, first),
                ..acc
            },
            ThenFmtRule::Next(next) => ThenFmtRules {
                b_rules: <B as FmtRule>::fill(acc.b_rules, next),
                ..acc
            },
        }
    }
}

pub struct ThenFmtRules<A, B> {
    a_rules: A,
    b_rules: B,
}

impl<A, B> FromDefaults<ThenFmtRule<A, B>>
    for ThenFmtRules<<A as FmtRule>::Struct, <B as FmtRule>::Struct>
where
    A: FmtRule,
    B: FmtRule,
{
    fn from_defaults() -> Self {
        Self {
            a_rules: <A as FmtRule>::Struct::from_defaults(),
            b_rules: <B as FmtRule>::Struct::from_defaults(),
        }
    }
}

impl<A, B, O> Promptable for Then<A, B, O>
where
    A: Promptable,
    B: Promptable,
    O: FromOutput<<Self as Flattenable>::RawOutput>,
{
    type Output = O;
    type FmtRule = ThenFmtRule<<A as Promptable>::FmtRule, <B as Promptable>::FmtRule>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<O>>
    where
        R: BufRead,
        W: Write,
    {
        let mut b_fmt = PromptFormat::default();
        let a_fmt = fmt
            .rules
            .iter()
            .copied()
            .filter_map(|rule| match rule {
                ThenFmtRule::First(a) => Some(a),
                ThenFmtRule::Next(b) => {
                    b_fmt.extend(Some(b));
                    None
                }
            })
            .collect();

        prompt_twice(read, write, self, &a_fmt, &b_fmt)
    }
}

impl<A, B, O> Promptable for ThenFmt<A, B, O>
where
    A: Promptable,
    B: Promptable<FmtRule = <A as Promptable>::FmtRule>,
    O: FromOutput<<Then<A, B, O> as Flattenable>::RawOutput>,
{
    type Output = O;
    type FmtRule = <A as Promptable>::FmtRule;

    #[inline(always)]
    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        prompt_twice(read, write, &mut self.inner, fmt, fmt)
    }
}

fn prompt_twice<R, W, A, B, O>(
    mut read: R, mut write: W, prompt: &mut Then<A, B, O>,
    first_fmt: &PromptFormat<<A as Promptable>::FmtRule>,
    then_fmt: &PromptFormat<<B as Promptable>::FmtRule>,
) -> io::Result<ControlFlow<O>>
where
    R: BufRead,
    W: Write,
    A: Promptable,
    B: Promptable,
    O: FromOutput<<Then<A, B, O> as Flattenable>::RawOutput>,
{
    let ControlFlow::Break(a) = prompt.first.prompt_once(&mut read, &mut write, first_fmt)? else {
        return Ok(ControlFlow::Continue(()));
    };

    let b = loop {
        if let ControlFlow::Break(b) = prompt.then.prompt_once(&mut read, &mut write, then_fmt)? {
            break b;
        }
    };

    Ok(ControlFlow::Break(FromOutput::from_output((a, b))))
}

#[derive(thiserror::Error, Debug)]
#[error("max tries exceeded")]
pub struct MaxTriesExceeded;

impl<P> Promptable for MaxTries<P>
where
    P: Promptable,
{
    type Output = Result<<P as Promptable>::Output, MaxTriesExceeded>;
    type FmtRule = <P as Promptable>::FmtRule;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
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

struct WrittenInner<'a, 'fmt> {
    msg: Option<&'a str>,
    _marker: PhantomData<&'fmt ()>,
}

fmt_rules! {
    enum WrittenFmtRule<'a> for struct WrittenFmtRules {
        Prefix(&'a str = ">> ") for prefix,
        BreakLine(bool = true) for break_line,
        RepeatPrompt(bool = false) for repeat_prompt,
    }
}

impl<'a> WrittenInner<'a, '_> {
    fn new(msg: &'a str) -> Self {
        Self {
            msg: Some(msg),
            _marker: PhantomData,
        }
    }

    fn prompt<R, W>(
        &mut self, mut read: R, mut write: W, fmt: &PromptFormat<WrittenFmtRule<'_>>,
    ) -> io::Result<String>
    where
        R: BufRead,
        W: Write,
    {
        let fmt = fmt.to_struct();
        if let Some(msg) = if fmt.repeat_prompt {
            self.msg
        } else {
            self.msg.take()
        } {
            write!(write, "{msg}")?;

            if fmt.break_line {
                writeln!(write)?;
            }
        }

        write!(write, "{}", fmt.prefix)?;
        write.flush()?;

        let mut s = String::new();
        read.read_line(&mut s)?;

        Ok(s.trim().to_owned())
    }
}

pub struct Bool<'a, 'fmt> {
    inner: WrittenInner<'a, 'fmt>,
}

pub fn bool(msg: &str) -> Bool<'_, '_> {
    Bool {
        inner: WrittenInner::new(msg),
    }
}

const TRUE_INPUTS: &[&str] = &["yes", "yep", "true"];
const FALSE_INPUTS: &[&str] = &["noppe", "nah", "false"];

impl<'fmt> Promptable for Bool<'_, 'fmt> {
    type Output = bool;
    type FmtRule = WrittenFmtRule<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let input = self.inner.prompt(read, write, fmt)?;
        Ok(match () {
            _ if TRUE_INPUTS.iter().any(|s| s.contains(&input)) => ControlFlow::Break(true),
            _ if FALSE_INPUTS.iter().any(|s| s.contains(&input)) => ControlFlow::Break(false),
            _ => ControlFlow::Continue(()),
        })
    }
}

pub struct Written<'a, 'fmt, T> {
    inner: WrittenInner<'a, 'fmt>,
    _marker: PhantomData<T>,
}

pub fn written<T>(msg: &str) -> Written<'_, '_, T> {
    Written {
        inner: WrittenInner::new(msg),
        _marker: PhantomData,
    }
}

impl<'fmt, T> Promptable for Written<'_, 'fmt, T>
where
    T: FromStr,
{
    type Output = T;
    type FmtRule = WrittenFmtRule<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let input = self.inner.prompt(read, write, fmt)?;
        match input.parse() {
            Ok(out) => Ok(ControlFlow::Break(out)),
            Err(_) => Ok(ControlFlow::Continue(())),
        }
    }
}

pub struct Selected<'a, 'fmt, const N: usize, T> {
    msgs: Option<[&'a str; N]>,
    values: [Option<T>; N],
    _marker: PhantomData<&'fmt ()>,
}

fmt_rules! {
    enum SelectedFmtRule<'a> for struct SelectedFmtRules {
        Prefix(&'a str = ">> ") for prefix,
    }
}

impl<'fmt, const N: usize, T> Promptable for Selected<'_, 'fmt, N, T> {
    type Output = T;
    type FmtRule = SelectedFmtRule<'fmt>;

    fn prompt_once<R, W>(
        &mut self, mut read: R, mut write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let fmt = fmt.to_struct();
        if let Some(list) = self.msgs.take() {
            for (msg, i) in list.into_iter().zip(1..) {
                writeln!(write, "[{i}] - {msg}")?;
            }
        }
        write!(write, "{}", fmt.prefix)?;
        write.flush()?;

        let mut s = String::new();
        read.read_line(&mut s)?;
        let i = match s.trim().parse::<usize>() {
            Ok(i) if i >= 1 && i <= self.values.len() => i,
            _ => return Ok(ControlFlow::Continue(())),
        };

        match self.values[i - 1].take() {
            Some(out) => Ok(ControlFlow::Break(out)),
            None => Ok(ControlFlow::Continue(())),
        }
    }
}

pub fn selected<const N: usize, T>(list: [(&str, T); N]) -> Selected<'_, '_, N, T> {
    fn split<const N: usize, A, B>(arr: [(A, B); N]) -> ([A; N], [B; N]) {
        use std::array::from_fn;
        let mut arr = arr.map(|(a, b)| (Some(a), Some(b)));
        let a = from_fn(|i| arr[i].0.take().unwrap());
        let b = from_fn(|i| arr[i].1.take().unwrap());
        (a, b)
    }

    let (msgs, values) = split(list.map(|(a, b)| (a, Some(b))));

    Selected {
        msgs: Some(msgs),
        values,
        _marker: PhantomData,
    }
}

pub struct Separated<'a, 'fmt, I, T> {
    inner: WrittenInner<'a, 'fmt>,
    sep: &'a str,
    _marker: PhantomData<(I, T)>,
}

impl<'fmt, I, T> Promptable for Separated<'_, 'fmt, I, T>
where
    I: FromIterator<T>,
    T: FromStr,
{
    type Output = I;
    type FmtRule = WrittenFmtRule<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        self.inner.prompt(read, write, fmt).map(|out| {
            match out
                .split(self.sep)
                .map(str::parse)
                .collect::<Result<I, _>>()
            {
                Ok(out) => ControlFlow::Break(out),
                Err(_) => ControlFlow::Continue(()),
            }
        })
    }
}

pub fn separated<'a, 'fmt, I, T>(msg: &'a str, sep: &'a str) -> Separated<'a, 'fmt, I, T> {
    Separated {
        inner: WrittenInner::new(msg),
        sep,
        _marker: PhantomData,
    }
}

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

trait TryFromOutput<Output>: Sized {
    fn try_from_output(output: Output) -> Option<Self>;
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
    type FmtRule = WrittenFmtRule<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &PromptFormat<Self::FmtRule>,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
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

/// Not public API.
#[doc(hidden)]
pub mod __private {
    /// Used in macros to quote `1`s
    pub trait _1 {
        const _1: usize;
    }

    impl<T> _1 for T {
        const _1: usize = 1;
    }

    pub use std::mem::discriminant;
}
