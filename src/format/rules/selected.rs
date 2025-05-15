use crate::format::{
    BreakLine, Fmt, InputPrefix, ListMsgPos, ListSurrounds, Mergeable, MsgPrefix, Position,
    RepeatPrompt, Unwrappable,
};

use super::UnwrappedWrittenFmtRules;

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
    pub msg_prefix: &'a str,
    pub input_prefix: &'a str,
    pub break_line: bool,
    pub repeat_prompt: bool,
    pub list_surrounds: (&'a str, &'a str),
    pub list_msg_pos: Position,
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
