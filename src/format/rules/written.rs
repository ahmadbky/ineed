use crate::format::{BreakLine, Fmt, InputPrefix, Mergeable, MsgPrefix, RepeatPrompt, Unwrappable};

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
    pub msg_prefix: &'a str,
    pub input_prefix: &'a str,
    pub break_line: bool,
    pub repeat_prompt: bool,
}

impl UnwrappedWrittenFmtRules<'_> {
    pub const DEFAULT: Self = Self {
        msg_prefix: "- ",
        input_prefix: "> ",
        break_line: true,
        repeat_prompt: false,
    };
}
