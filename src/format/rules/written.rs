use crate::format::{
    BreakLine, ConstDefault, Fmt, InputPrefix, Mergeable, MsgPrefix, Partial, RepeatPrompt,
};

/// The set of rules accepted by written prompts (e.g. with [`written`](crate::written), etc).
///
/// See the [module documentation](crate::format) for more information.
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

impl<'a> Partial for WrittenFmtRules<'a> {
    type Expanded = ExpandedWrittenFmtRules<'a>;

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
        }
    }
}

/// Expanded version of [`WrittenFmtRules`].
#[derive(Debug, PartialEq, Eq)]
pub struct ExpandedWrittenFmtRules<'a> {
    /// The message prefix, that is put right before the message.
    pub msg_prefix: &'a str,
    /// The input prefix, that is put right before the user input.
    pub input_prefix: &'a str,
    /// Whether to break the line after the message or not.
    pub break_line: bool,
    /// Whether to repeat the message, along with its prefix and the input prefix,
    /// if the previous input is invalid. If not, only the input prefix is repeated.
    pub repeat_prompt: bool,
}

impl ConstDefault for ExpandedWrittenFmtRules<'_> {
    const DEFAULT: Self = Self {
        msg_prefix: "- ",
        input_prefix: "> ",
        break_line: true,
        repeat_prompt: false,
    };
}

impl Default for ExpandedWrittenFmtRules<'_> {
    #[inline(always)]
    fn default() -> Self {
        Self::DEFAULT
    }
}
