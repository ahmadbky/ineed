//! Module exposing types to customize prompt styling.
//!
//! # How prompts format are customized
//!
//! The display of prompts can be customized with format rules. To customize a prompt format,
//! you can use the [`Promptable::fmt`](crate::Promptable::fmt) method.
//!
//! Each promptable type specifies the set of format rules it accepts, with the
//! [`Promptable::FmtRules`][crate::Promptable::FmtRules] associated type. For example, the
//! written inputs (e.g. with [`written`](crate::written), [`bool`](crate::bool), etc) accept
//! the set of rules [`WrittenFmtRules`][1].
//!
//! A set of rules is a type that implements `From<...>` for each format rule it supports.
//! For example, [`WrittenFmtRules`][1] implements `From<`[`InputPrefix`]`>`, which means you can
//! provide a custom input prefix for written input prompts, by calling
//! `ineed::fmt().`[`input_prefix(...)`](FmtRule::input_prefix).
//!
//! You can also find the *expanded* version of the set of rules applied to a prompt. For example:
//! [`ExpandedWrittenFmtRules`](rules::ExpandedWrittenFmtRules). This is explained in the next part.
//!
//! # How format customization works
//!
//! When building a promptable (i.e. before calling any prompt method from the
//! [`Promptable`](crate::Promptable) trait), the format rules either don't exist (if you didn't
//! provide any custom format rules) or are still in their partial forms (which are represented
//! by the [`Promptable::FmtRules`](crate::Promptable::FmtRules) associated type).
//!
//! When calling any prompt method, the partial form of the format rules is expanded into
//! a struct that contains all the rules that the prompt is going to use. This struct is represented
//! by the [`Partial::Expanded`] associated type.
//!
//! The rules expansion is made so that every omitted format rule takes its default value (i.e. with
//! the [`ConstDefault::DEFAULT`] associated constant). Moreover, when combining promptables,
//! it merges the format rules of these promptables into one expanded form.
//!
//! If, during the merge, any format rule conflicts, the chosen format rule is the one defined
//! closest to the promptables in question. For example here:
//!
//! ```no_run
//! # use ineed::prelude::*;
//! let age = ineed::written::<u8>("Your age")
//!   .fmt(ineed::fmt().input_prefix(">> "))
//!   .until(|age| *age < 120)
//!   .fmt(ineed::fmt().input_prefix("=> "))
//!   .prompt()
//!   .unwrap();
//! ```
//!
//! In such case, the input prefix used is `>> `, as it's the one defined closest
//! to the written promptable.
//!
//! There is a similar case when [chaining promptables](crate::Promptable#prompt-format).
//!
//! [1]: rules::WrittenFmtRules

pub mod rules;

/// The base type to customize a prompt styling.
///
/// This is intended to be used with the [`fmt()`] function.
#[derive(Clone, Copy)]
pub struct Fmt(());

impl FmtRule for Fmt {}

/// Base function to start customizing the prompt format.
///
/// Calling this function alone does nothing. It is intended to be chained with calls to the
/// [`FmtRule`] trait, as a parameter of the [`Promptable::fmt`](crate::Promptable::fmt) method.
///
/// See the [module documentation](self) for more information.
#[inline(always)]
pub fn fmt() -> Fmt {
    Fmt(())
}

/// Represents a type of value that can merge with another value to produce a new value.
///
/// This is implemented for set of rules, represented by the [`FmtRules`] trait.
pub trait Mergeable {
    /// Merges this value with the other, and returns the result of the merge.
    fn merge_with(&self, other: &Self) -> Self;
}

/// Represents a type that can transform from a partial form into an expanded form.
///
/// This is implemented for set of rules, represented by the [`FmtRules`] trait.
///
/// See the [module documentation](self) for more information.
pub trait Partial {
    /// The type of the expanded form.
    type Expanded: ConstDefault;

    /// Transform the partial form into the expanded form.
    fn expand(&self) -> Self::Expanded;
}

/// Const-version of the [`Default`] trait.
pub trait ConstDefault {
    /// The default value of the type.
    const DEFAULT: Self;
}

/// Represents a set of format rules.
///
/// They're intended to be chained with the methods of this trait.
pub trait FmtRule: Sized + Copy {
    /// The message prefix, usually put right before the message.
    fn msg_prefix(self, prefix: &str) -> MsgPrefix<'_, Self> {
        MsgPrefix { rule: self, prefix }
    }

    /// The input prefix, usually put right before the user input.
    fn input_prefix(self, prefix: &str) -> InputPrefix<'_, Self> {
        InputPrefix { rule: self, prefix }
    }

    /// Represents the surrounds of the index of each list item for selectable prompts.
    fn list_surrounds<'a>(self, open: &'a str, close: &'a str) -> ListSurrounds<'a, Self> {
        ListSurrounds {
            rule: self,
            surrounds: (open, close),
        }
    }

    /// The position of the message for selectable prompts (either below or above the list).
    fn list_msg_pos(self, pos: Position) -> ListMsgPos<Self> {
        ListMsgPos { rule: self, pos }
    }

    /// Whether to break a line right after the message or not.
    fn break_line(self, value: bool) -> BreakLine<Self> {
        BreakLine { rule: self, value }
    }

    /// Whether to repeat the message along with its prefix and the input prefix, when the previous
    /// input is invalid, or not.
    ///
    /// If this is false, only the input prefix is repeated in such case.
    fn repeat_prompt(self, value: bool) -> RepeatPrompt<Self> {
        RepeatPrompt { rule: self, value }
    }
}

/// The message prefix format rule, usually put right before the message.
///
/// This is returned by [`FmtRule::msg_prefix`].
#[derive(Clone, Copy)]
pub struct MsgPrefix<'a, R> {
    pub(crate) rule: R,
    pub(crate) prefix: &'a str,
}

impl<R: FmtRule> FmtRule for MsgPrefix<'_, R> {}

/// The input prefix format rule, usually put right before the user input.
///
/// This is returned by [`FmtRule::input_prefix`].
#[derive(Clone, Copy)]
pub struct InputPrefix<'a, R> {
    pub(crate) rule: R,
    pub(crate) prefix: &'a str,
}

impl<R: FmtRule> FmtRule for InputPrefix<'_, R> {}

/// The format rule of the surrounds of the index of each list item for selectable prompts.
///
/// This is returned by [`FmtRule::list_surrounds`].
#[derive(Clone, Copy)]
pub struct ListSurrounds<'a, R> {
    pub(crate) rule: R,
    pub(crate) surrounds: (&'a str, &'a str),
}

impl<R: FmtRule> FmtRule for ListSurrounds<'_, R> {}

/// The position of the message, e.g. for selectable prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// The message is displayed on the top (e.g. above the list for selectable prompts).
    Top,
    /// The message is displayed on the bottom (e.g. below the list for selectable prompts).
    Bottom,
}

/// The format rule of the message position, e.g. for selectable prompts.
///
/// This is returned by [`FmtRule::list_msg_pos`].
#[derive(Clone, Copy)]
pub struct ListMsgPos<R> {
    pub(crate) rule: R,
    pub(crate) pos: Position,
}

impl<R: FmtRule> FmtRule for ListMsgPos<R> {}

/// The format rule to whether break a line or not right after the message.
///
/// This is returned by [`FmtRule::break_line`].
#[derive(Clone, Copy)]
pub struct BreakLine<R> {
    pub(crate) rule: R,
    pub(crate) value: bool,
}

impl<R: FmtRule> FmtRule for BreakLine<R> {}

/// The format rule to whether repeat the message along with its prefix and input prefix.
///
/// This is returned by [`FmtRule::repeat_prompt`].
#[derive(Clone, Copy)]
pub struct RepeatPrompt<R> {
    pub(crate) rule: R,
    pub(crate) value: bool,
}

impl<R: FmtRule> FmtRule for RepeatPrompt<R> {}

/// Types representing set of rules supported by promptables.
///
/// This is used as a bound for the [`Promptable::FmtRules`](crate::Promptable::FmtRules)
/// associated type.
#[cfg_attr(nightly, doc(notable_trait))]
pub trait FmtRules: From<Fmt> + Mergeable + Partial + Default {}
impl<T> FmtRules for T where T: From<Fmt> + Mergeable + Partial + Default {}

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
        Partial as _, Position,
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
