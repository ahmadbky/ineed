#![cfg_attr(nightly, feature(doc_cfg))]

use std::{
    io::{self, BufRead, Write},
    marker::PhantomData,
    ops::ControlFlow,
    str::FromStr,
};

use self::format::{
    BreakLine, Fmt, FmtRule, FmtRules, InputPrefix, ListMsgPos, ListSurrounds, Mergeable,
    MsgPrefix, Position, RepeatPrompt, Unwrappable,
};

pub mod format;
pub use format::fmt;

pub mod prelude {
    pub use super::{Promptable as _, format::FmtRule as _};
}

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

pub struct Formatted<P: Promptable> {
    prompt: P,
    rules: <P as Promptable>::FmtRules,
}

pub trait Promptable: Sized {
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

    fn fmt<F>(self, fmt: F) -> Formatted<Self>
    where
        Self::FmtRules: From<F>,
    {
        Formatted {
            prompt: self,
            rules: fmt.into(),
        }
    }
}

impl<P: Promptable> Promptable for Formatted<P> {
    type Output = <P as Promptable>::Output;
    type FmtRules = <P as Promptable>::FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let fmt = self.rules.merge_with(fmt);
        self.prompt.prompt_once(read, write, &fmt)
    }
}

impl<P, F, T> Promptable for Map<P, F>
where
    P: Promptable,
    F: FnMut(<P as Promptable>::Output) -> T,
{
    type Output = T;
    type FmtRules = <P as Promptable>::FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
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
    type FmtRules = <P as Promptable>::FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
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

pub struct ThenFmtRules<A, B> {
    a_rules: A,
    b_rules: B,
}

impl<A, B, R> From<R> for ThenFmtRules<A, B>
where
    A: From<R>,
    B: From<R>,
    R: FmtRule,
{
    fn from(value: R) -> Self {
        Self {
            a_rules: A::from(value),
            b_rules: B::from(value),
        }
    }
}

impl<A, B> Mergeable for ThenFmtRules<A, B>
where
    A: Mergeable,
    B: Mergeable,
{
    fn merge_with(&self, other: &Self) -> Self {
        Self {
            a_rules: self.a_rules.merge_with(&other.a_rules),
            b_rules: self.b_rules.merge_with(&other.b_rules),
        }
    }
}

impl<A, B> Unwrappable for ThenFmtRules<A, B> {
    type Unwrapped = ();
    fn unwrap(&self) -> Self::Unwrapped {}
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
        R: BufRead,
        W: Write,
    {
        prompt_twice(read, write, self, fmt)
    }
}

fn prompt_twice<R, W, A, B, O>(
    mut read: R, mut write: W, prompt: &mut Then<A, B, O>,
    fmt: &<Then<A, B, O> as Promptable>::FmtRules,
) -> io::Result<ControlFlow<O>>
where
    R: BufRead,
    W: Write,
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

#[derive(thiserror::Error, Debug)]
#[error("max tries exceeded")]
pub struct MaxTriesExceeded;

impl<P> Promptable for MaxTries<P>
where
    P: Promptable,
{
    type Output = Result<<P as Promptable>::Output, MaxTriesExceeded>;
    type FmtRules = <P as Promptable>::FmtRules;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
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

#[derive(Default)]
pub struct WrittenFmtRules<'a> {
    msg_prefix: Option<&'a str>,
    input_prefix: Option<&'a str>,
    break_line: Option<bool>,
    repeat_prompt: Option<bool>,
}

impl<'a, R> From<MsgPrefix<'a, R>> for WrittenFmtRules<'a>
where
    Self: From<R>,
{
    fn from(value: MsgPrefix<'a, R>) -> Self {
        Self {
            msg_prefix: Some(value.prefix),
            ..Self::from(value.rule)
        }
    }
}

impl<'a, R> From<InputPrefix<'a, R>> for WrittenFmtRules<'a>
where
    Self: From<R>,
{
    fn from(value: InputPrefix<'a, R>) -> Self {
        Self {
            input_prefix: Some(value.prefix),
            ..Self::from(value.rule)
        }
    }
}

impl<R> From<BreakLine<R>> for WrittenFmtRules<'_>
where
    Self: From<R>,
{
    fn from(value: BreakLine<R>) -> Self {
        Self {
            break_line: Some(value.value),
            ..Self::from(value.rule)
        }
    }
}

impl<R> From<RepeatPrompt<R>> for WrittenFmtRules<'_>
where
    Self: From<R>,
{
    fn from(value: RepeatPrompt<R>) -> Self {
        Self {
            repeat_prompt: Some(value.value),
            ..Self::from(value.rule)
        }
    }
}

impl From<Fmt> for WrittenFmtRules<'_> {
    fn from(_: Fmt) -> Self {
        Self::default()
    }
}

impl Mergeable for WrittenFmtRules<'_> {
    fn merge_with(&self, other: &Self) -> Self {
        Self {
            msg_prefix: self.msg_prefix.or(other.msg_prefix),
            input_prefix: self.input_prefix.or(other.input_prefix),
            break_line: self.break_line.or(other.break_line),
            repeat_prompt: self.repeat_prompt.or(other.repeat_prompt),
        }
    }
}

impl<'a> Unwrappable for WrittenFmtRules<'a> {
    type Unwrapped = UnwrappedWrittenFmtRules<'a>;

    fn unwrap(&self) -> Self::Unwrapped {
        Self::Unwrapped {
            msg_prefix: self
                .msg_prefix
                .unwrap_or(Self::Unwrapped::DEFAULT.msg_prefix),
            input_prefix: self
                .input_prefix
                .unwrap_or(Self::Unwrapped::DEFAULT.input_prefix),
            break_line: self
                .break_line
                .unwrap_or(Self::Unwrapped::DEFAULT.break_line),
            repeat_prompt: self
                .repeat_prompt
                .unwrap_or(Self::Unwrapped::DEFAULT.repeat_prompt),
        }
    }
}

pub struct UnwrappedWrittenFmtRules<'a> {
    msg_prefix: &'a str,
    input_prefix: &'a str,
    break_line: bool,
    repeat_prompt: bool,
}

impl UnwrappedWrittenFmtRules<'_> {
    pub const DEFAULT: Self = Self {
        msg_prefix: "- ",
        input_prefix: "> ",
        break_line: true,
        repeat_prompt: false,
    };
}

struct WrittenInner<'a, 'fmt> {
    msg: Option<&'a str>,
    _marker: PhantomData<&'fmt ()>,
}

impl<'a> WrittenInner<'a, '_> {
    fn new(msg: &'a str) -> Self {
        Self {
            msg: Some(msg),
            _marker: PhantomData,
        }
    }

    fn prompt_with<R, W, F>(
        &mut self, mut read: R, mut write: W, fmt: &WrittenFmtRules<'_>, f: F,
    ) -> io::Result<String>
    where
        R: BufRead,
        W: Write,
        F: FnOnce(&mut R) -> io::Result<String>,
    {
        let fmt = fmt.unwrap();

        if let Some(msg) = if fmt.repeat_prompt {
            self.msg
        } else {
            self.msg.take()
        } {
            write!(write, "{}{msg}", fmt.msg_prefix)?;

            if fmt.break_line {
                writeln!(write)?;
            }
        }

        write!(write, "{}", fmt.input_prefix)?;
        write.flush()?;

        Ok(f(&mut read)?.trim().to_owned())
    }

    fn prompt<R, W>(&mut self, read: R, write: W, fmt: &WrittenFmtRules<'_>) -> io::Result<String>
    where
        R: BufRead,
        W: Write,
    {
        self.prompt_with(read, write, fmt, |read| {
            let mut s = String::new();
            read.read_line(&mut s)?;
            Ok(s)
        })
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

const TRUE_INPUTS: &[&str] = &["y", "ye", "yes", "yep", "true"];
const FALSE_INPUTS: &[&str] = &["n", "no", "nop", "nope", "nopp", "nah", "false"];

impl<'fmt> Promptable for Bool<'_, 'fmt> {
    type Output = bool;
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let input = self.inner.prompt(read, write, fmt)?.trim().to_lowercase();
        Ok(match () {
            _ if TRUE_INPUTS.iter().any(|s| input.as_str() == *s) => ControlFlow::Break(true),
            _ if FALSE_INPUTS.iter().any(|s| input.as_str() == *s) => ControlFlow::Break(false),
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
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let input = self.inner.prompt(read, write, fmt)?;
        match input.parse() {
            Ok(out) if !input.is_empty() => Ok(ControlFlow::Break(out)),
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

pub struct Selected<'a, 'fmt, const N: usize, T> {
    title: Option<&'a str>,
    msgs: Option<[&'a str; N]>,
    values: [Option<T>; N],
    _marker: PhantomData<&'fmt ()>,
}

#[derive(Default)]
pub struct SelectedFmtRules<'a> {
    msg_prefix: Option<&'a str>,
    input_prefix: Option<&'a str>,
    repeat_prompt: Option<bool>,
    break_line: Option<bool>,
    list_surrounds: Option<(&'a str, &'a str)>,
    list_msg_pos: Option<Position>,
}

impl From<Fmt> for SelectedFmtRules<'_> {
    fn from(_: Fmt) -> Self {
        Self::default()
    }
}

impl<'a, R> From<MsgPrefix<'a, R>> for SelectedFmtRules<'a>
where
    Self: From<R>,
{
    fn from(value: MsgPrefix<'a, R>) -> Self {
        Self {
            msg_prefix: Some(value.prefix),
            ..Self::from(value.rule)
        }
    }
}

impl<'a, R> From<InputPrefix<'a, R>> for SelectedFmtRules<'a>
where
    Self: From<R>,
{
    fn from(value: InputPrefix<'a, R>) -> Self {
        Self {
            input_prefix: Some(value.prefix),
            ..Self::from(value.rule)
        }
    }
}

impl<R> From<BreakLine<R>> for SelectedFmtRules<'_>
where
    Self: From<R>,
{
    fn from(value: BreakLine<R>) -> Self {
        Self {
            break_line: Some(value.value),
            ..Self::from(value.rule)
        }
    }
}

impl<R> From<RepeatPrompt<R>> for SelectedFmtRules<'_>
where
    Self: From<R>,
{
    fn from(value: RepeatPrompt<R>) -> Self {
        Self {
            repeat_prompt: Some(value.value),
            ..Self::from(value.rule)
        }
    }
}

impl<R> From<ListMsgPos<R>> for SelectedFmtRules<'_>
where
    Self: From<R>,
{
    fn from(value: ListMsgPos<R>) -> Self {
        Self {
            list_msg_pos: Some(value.pos),
            ..Self::from(value.rule)
        }
    }
}

impl<'a, R> From<ListSurrounds<'a, R>> for SelectedFmtRules<'a>
where
    Self: From<R>,
{
    fn from(value: ListSurrounds<'a, R>) -> Self {
        Self {
            list_surrounds: Some(value.surrounds),
            ..Self::from(value.rule)
        }
    }
}

impl Mergeable for SelectedFmtRules<'_> {
    fn merge_with(&self, other: &Self) -> Self {
        Self {
            msg_prefix: self.msg_prefix.or(other.msg_prefix),
            input_prefix: self.input_prefix.or(other.input_prefix),
            break_line: self.break_line.or(other.break_line),
            repeat_prompt: self.repeat_prompt.or(other.repeat_prompt),
            list_surrounds: self.list_surrounds.or(other.list_surrounds),
            list_msg_pos: self.list_msg_pos.or(other.list_msg_pos),
        }
    }
}

impl<'a> Unwrappable for SelectedFmtRules<'a> {
    type Unwrapped = UnwrappedSelectedFmtRules<'a>;

    fn unwrap(&self) -> Self::Unwrapped {
        Self::Unwrapped {
            msg_prefix: self
                .msg_prefix
                .unwrap_or(Self::Unwrapped::DEFAULT.msg_prefix),
            input_prefix: self
                .input_prefix
                .unwrap_or(Self::Unwrapped::DEFAULT.input_prefix),
            break_line: self
                .break_line
                .unwrap_or(Self::Unwrapped::DEFAULT.break_line),
            repeat_prompt: self
                .repeat_prompt
                .unwrap_or(Self::Unwrapped::DEFAULT.repeat_prompt),
            list_surrounds: self
                .list_surrounds
                .unwrap_or(Self::Unwrapped::DEFAULT.list_surrounds),
            list_msg_pos: self
                .list_msg_pos
                .unwrap_or(Self::Unwrapped::DEFAULT.list_msg_pos),
        }
    }
}

pub struct UnwrappedSelectedFmtRules<'a> {
    msg_prefix: &'a str,
    input_prefix: &'a str,
    break_line: bool,
    repeat_prompt: bool,
    list_surrounds: (&'a str, &'a str),
    list_msg_pos: Position,
}

impl UnwrappedSelectedFmtRules<'_> {
    pub const DEFAULT: Self = Self {
        msg_prefix: UnwrappedWrittenFmtRules::DEFAULT.msg_prefix,
        input_prefix: UnwrappedWrittenFmtRules::DEFAULT.input_prefix,
        break_line: UnwrappedWrittenFmtRules::DEFAULT.break_line,
        repeat_prompt: UnwrappedWrittenFmtRules::DEFAULT.repeat_prompt,
        list_surrounds: ("[", "]"),
        list_msg_pos: Position::Bottom,
    };
}

impl<'fmt, const N: usize, T> Promptable for Selected<'_, 'fmt, N, T> {
    type Output = T;
    type FmtRules = SelectedFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, mut read: R, mut write: W, fmt: &Self::FmtRules,
    ) -> io::Result<ControlFlow<Self::Output>>
    where
        R: BufRead,
        W: Write,
    {
        let fmt = fmt.unwrap();
        let (open, close) = fmt.list_surrounds;

        if let Position::Top = fmt.list_msg_pos {
            if let Some(title) = if fmt.repeat_prompt {
                self.title
            } else {
                self.title.take()
            } {
                writeln!(write, "{}{}", fmt.msg_prefix, title)?;
            }
        }
        if let Some(list) = self.msgs.take() {
            for (msg, i) in list.into_iter().zip(1..) {
                writeln!(write, "{open}{i}{close} - {msg}")?;
            }
        }
        if let Position::Bottom = fmt.list_msg_pos {
            if let Some(title) = if fmt.repeat_prompt {
                self.title
            } else {
                self.title.take()
            } {
                write!(write, "{}{}", fmt.msg_prefix, title)?;
                if fmt.break_line {
                    writeln!(write)?;
                }
            }
        }

        write!(write, "{}", fmt.input_prefix)?;
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

pub fn selected<'a, 'fmt, const N: usize, T>(
    title: &'a str, list: [(&'a str, T); N],
) -> Selected<'a, 'fmt, N, T> {
    fn split<const N: usize, A, B>(arr: [(A, B); N]) -> ([A; N], [B; N]) {
        use std::array::from_fn;
        let mut arr = arr.map(|(a, b)| (Some(a), Some(b)));
        let a = from_fn(|i| arr[i].0.take().unwrap());
        let b = from_fn(|i| arr[i].1.take().unwrap());
        (a, b)
    }

    let (msgs, values) = split(list.map(|(a, b)| (a, Some(b))));

    Selected {
        title: Some(title),
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
    type FmtRules = WrittenFmtRules<'fmt>;

    fn prompt_once<R, W>(
        &mut self, read: R, write: W, fmt: &Self::FmtRules,
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
                Ok(o) if !out.is_empty() => ControlFlow::Break(o),
                _ => ControlFlow::Continue(()),
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
        R: BufRead,
        W: Write,
    {
        self.inner
            .prompt_with(read, write, fmt, |_| rpassword::read_password())
            .map(|s| match s.is_empty() {
                true => ControlFlow::Continue(()),
                false => ControlFlow::Break(s),
            })
    }
}
