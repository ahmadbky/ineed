#[derive(Clone, Copy)]
pub struct Fmt;

impl FmtRule for Fmt {}

#[inline(always)]
pub fn fmt() -> Fmt {
    Fmt
}

pub trait Mergeable {
    fn merge_with(&self, other: &Self) -> Self;
}

pub trait Unwrappable {
    type Unwrapped;

    fn unwrap(&self) -> Self::Unwrapped;
}

pub trait FmtRule: Sized + Copy {
    fn msg_prefix(self, prefix: &str) -> MsgPrefix<'_, Self> {
        MsgPrefix { rule: self, prefix }
    }

    fn input_prefix(self, prefix: &str) -> InputPrefix<'_, Self> {
        InputPrefix { rule: self, prefix }
    }

    fn list_surrounds<'a>(self, open: &'a str, close: &'a str) -> ListSurrounds<'a, Self> {
        ListSurrounds {
            rule: self,
            surrounds: (open, close),
        }
    }

    fn list_msg_pos(self, pos: Position) -> ListMsgPos<Self> {
        ListMsgPos {
            rule: self,
            pos,
        }
    }

    fn break_line(self, value: bool) -> BreakLine<Self> {
        BreakLine { rule: self, value }
    }

    fn repeat_prompt(self, value: bool) -> RepeatPrompt<Self> {
        RepeatPrompt { rule: self, value }
    }
}

#[derive(Clone, Copy)]
pub struct MsgPrefix<'a, R> {
    pub(crate) rule: R,
    pub(crate) prefix: &'a str,
}

impl<R: FmtRule> FmtRule for MsgPrefix<'_, R> {}

#[derive(Clone, Copy)]
pub struct InputPrefix<'a, R> {
    pub(crate) rule: R,
    pub(crate) prefix: &'a str,
}

impl<R: FmtRule> FmtRule for InputPrefix<'_, R> {}

#[derive(Clone, Copy)]
pub struct ListSurrounds<'a, R> {
    pub(crate) rule: R,
    pub(crate) surrounds: (&'a str, &'a str),
}

impl<R: FmtRule> FmtRule for ListSurrounds<'_, R> {}

#[derive(Clone, Copy)]
pub enum Position {
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
pub struct ListMsgPos<R> {
    pub(crate) rule: R,
    pub(crate) pos: Position,
}

impl<R: FmtRule> FmtRule for ListMsgPos<R> {}

#[derive(Clone, Copy)]
pub struct BreakLine<R> {
    pub(crate) rule: R,
    pub(crate) value: bool,
}

impl<R: FmtRule> FmtRule for BreakLine<R> {}

#[derive(Clone, Copy)]
pub struct RepeatPrompt<R> {
    pub(crate) rule: R,
    pub(crate) value: bool,
}

impl<R: FmtRule> FmtRule for RepeatPrompt<R> {}

pub trait FmtRules: From<Fmt> + Mergeable + Unwrappable {}
impl<T> FmtRules for T where T: From<Fmt> + Mergeable + Unwrappable {}
