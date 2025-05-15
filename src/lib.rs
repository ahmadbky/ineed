#![cfg_attr(nightly, feature(doc_cfg))]

use std::{
    io::{self, BufRead, Write},
    marker::PhantomData,
    ops::ControlFlow,
};

use self::format::{
    BreakLine, Fmt, FmtRule, FmtRules, InputPrefix, ListMsgPos, ListSurrounds, Mergeable,
    MsgPrefix, Position, RepeatPrompt, Unwrappable,
};

pub mod format;
pub use format::fmt;

mod promptables;
pub use promptables::*;

pub mod prelude {
    pub use super::{Promptable as _, format::FmtRule as _};
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
