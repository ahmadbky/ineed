//! Module exposing types to customize prompt styling.

pub mod rules;

/// The base type to customize a prompt styling.
///
/// This is intended to be used with the [`fmt()`] function.
#[derive(Clone, Copy)]
pub struct Fmt {
    _priv: (),
}

impl FmtRule for Fmt {}

#[inline(always)]
pub fn fmt() -> Fmt {
    Fmt { _priv: () }
}

pub trait Mergeable {
    fn merge_with(&self, other: &Self) -> Self;
}

pub trait Expandable {
    type Expanded;

    fn expand(&self) -> Self::Expanded;
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
        ListMsgPos { rule: self, pos }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub trait FmtRules: From<Fmt> + Mergeable + Expandable + Default {}
impl<T> FmtRules for T where T: From<Fmt> + Mergeable + Expandable + Default {}

#[cfg(test)]
mod tests {
    use crate::{
        format::{
            Mergeable,
            rules::{ExpandedSelectedFmtRules, ExpandedWrittenFmtRules},
        },
        prelude::*,
    };

    use super::{
        Expandable as _, Position,
        rules::{SelectedFmtRules, WrittenFmtRules},
    };

    #[test]
    fn partial_written_fmt_infer_default() {
        let default_fmt_rules = WrittenFmtRules::default().expand();
        let fmt_rules = crate::fmt()
            .msg_prefix("my super prefix")
            .break_line(!default_fmt_rules.break_line);
        let fmt_rules = WrittenFmtRules::from(fmt_rules).expand();

        assert_eq!(
            fmt_rules,
            ExpandedWrittenFmtRules {
                msg_prefix: "my super prefix",
                break_line: !default_fmt_rules.break_line,
                ..default_fmt_rules
            }
        )
    }

    #[test]
    fn written_fmt_exclusive_merge() {
        let default_fmt_rules = WrittenFmtRules::default().expand();

        let fmt_rules1 = crate::fmt()
            .msg_prefix("my super prefix")
            .break_line(!default_fmt_rules.break_line);
        let fmt_rules1 = WrittenFmtRules::from(fmt_rules1);

        let fmt_rules2 = crate::fmt()
            .input_prefix("my giga input prefix")
            .repeat_prompt(!default_fmt_rules.repeat_prompt);
        let fmt_rules2 = WrittenFmtRules::from(fmt_rules2);

        let fmt_rules = fmt_rules1.merge_with(&fmt_rules2).expand();

        assert_eq!(
            fmt_rules,
            ExpandedWrittenFmtRules {
                msg_prefix: "my super prefix",
                input_prefix: "my giga input prefix",
                break_line: !default_fmt_rules.break_line,
                repeat_prompt: !default_fmt_rules.repeat_prompt,
            }
        )
    }

    #[test]
    fn written_fmt_conflicting_merge() {
        let fmt_rules1 = crate::fmt().msg_prefix("my msg prefix 1");
        let fmt_rules1 = WrittenFmtRules::from(fmt_rules1);

        let fmt_rules2 = crate::fmt().msg_prefix("my omega msg prefix 2");
        let fmt_rules2 = WrittenFmtRules::from(fmt_rules2);

        let merged = fmt_rules1.merge_with(&fmt_rules2).expand();
        assert_eq!(
            merged,
            ExpandedWrittenFmtRules {
                msg_prefix: "my msg prefix 1",
                ..Default::default()
            }
        );

        let merged = fmt_rules2.merge_with(&fmt_rules1).expand();
        assert_eq!(
            merged,
            ExpandedWrittenFmtRules {
                msg_prefix: "my omega msg prefix 2",
                ..Default::default()
            }
        );
    }

    #[test]
    fn partial_selected_fmt_infer_default() {
        let default_fmt_rules = SelectedFmtRules::default().expand();
        let fmt_rules = crate::fmt()
            .msg_prefix("my super prefix")
            .break_line(!default_fmt_rules.break_line)
            .list_msg_pos(Position::Bottom);
        let fmt_rules = SelectedFmtRules::from(fmt_rules).expand();

        assert_eq!(
            fmt_rules,
            ExpandedSelectedFmtRules {
                msg_prefix: "my super prefix",
                break_line: !default_fmt_rules.break_line,
                list_msg_pos: Position::Bottom,
                ..default_fmt_rules
            }
        );
    }

    #[test]
    fn selected_fmt_exclusive_merge() {
        let default_fmt_rules = SelectedFmtRules::default().expand();

        let fmt_rules1 = crate::fmt().list_surrounds("<", ">").input_prefix("-> ");
        let fmt_rules1 = SelectedFmtRules::from(fmt_rules1);

        let fmt_rules2 = crate::fmt()
            .msg_prefix("my giga msg prefix")
            .repeat_prompt(!default_fmt_rules.repeat_prompt);
        let fmt_rules2 = SelectedFmtRules::from(fmt_rules2);

        let fmt_rules = fmt_rules1.merge_with(&fmt_rules2).expand();

        assert_eq!(
            fmt_rules,
            ExpandedSelectedFmtRules {
                msg_prefix: "my giga msg prefix",
                input_prefix: "-> ",
                repeat_prompt: !default_fmt_rules.repeat_prompt,
                list_surrounds: ("<", ">"),
                ..default_fmt_rules
            }
        )
    }

    #[test]
    fn selected_fmt_conflicting_merge() {
        let fmt_rules1 = crate::fmt().list_surrounds("<", ">");
        let fmt_rules1 = SelectedFmtRules::from(fmt_rules1);

        let fmt_rules2 = crate::fmt().list_surrounds("<<", ">>");
        let fmt_rules2 = SelectedFmtRules::from(fmt_rules2);

        let merged = fmt_rules1.merge_with(&fmt_rules2).expand();

        assert_eq!(
            merged,
            ExpandedSelectedFmtRules {
                list_surrounds: ("<", ">"),
                ..Default::default()
            }
        );

        let merged = fmt_rules2.merge_with(&fmt_rules1).expand();

        assert_eq!(
            merged,
            ExpandedSelectedFmtRules {
                list_surrounds: ("<<", ">>"),
                ..Default::default()
            }
        );
    }
}
