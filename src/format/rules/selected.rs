use crate::format::{
    BreakLine, Expandable, Fmt, InputPrefix, ListMsgPos, ListSurrounds, Mergeable, MsgPrefix,
    Position, RepeatPrompt,
};

use super::ExpandedWrittenFmtRules;

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

impl<'a> Expandable for SelectedFmtRules<'a> {
    type Expanded = ExpandedSelectedFmtRules<'a>;

    fn expand(&self) -> Self::Expanded {
        Self::Expanded {
            msg_prefix: self
                .msg_prefix
                .unwrap_or(Self::Expanded::DEFAULT.msg_prefix),
            input_prefix: self
                .input_prefix
                .unwrap_or(Self::Expanded::DEFAULT.input_prefix),
            break_line: self
                .break_line
                .unwrap_or(Self::Expanded::DEFAULT.break_line),
            repeat_prompt: self
                .repeat_prompt
                .unwrap_or(Self::Expanded::DEFAULT.repeat_prompt),
            list_surrounds: self
                .list_surrounds
                .unwrap_or(Self::Expanded::DEFAULT.list_surrounds),
            list_msg_pos: self
                .list_msg_pos
                .unwrap_or(Self::Expanded::DEFAULT.list_msg_pos),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExpandedSelectedFmtRules<'a> {
    pub msg_prefix: &'a str,
    pub input_prefix: &'a str,
    pub break_line: bool,
    pub repeat_prompt: bool,
    pub list_surrounds: (&'a str, &'a str),
    pub list_msg_pos: Position,
}

impl ExpandedSelectedFmtRules<'_> {
    pub const DEFAULT: Self = Self {
        msg_prefix: ExpandedWrittenFmtRules::DEFAULT.msg_prefix,
        input_prefix: ExpandedWrittenFmtRules::DEFAULT.input_prefix,
        break_line: ExpandedWrittenFmtRules::DEFAULT.break_line,
        repeat_prompt: ExpandedWrittenFmtRules::DEFAULT.repeat_prompt,
        list_surrounds: ("[", "]"),
        list_msg_pos: Position::Bottom,
    };
}

impl Default for ExpandedSelectedFmtRules<'_> {
    fn default() -> Self {
        Self::DEFAULT
    }
}
