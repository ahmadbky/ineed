use std::{io, marker::PhantomData, ops::ControlFlow, str::FromStr};

use crate::{Promptable, WrittenFmtRules, format::Expandable as _};

pub(crate) struct WrittenInner<'a, 'fmt> {
    msg: Option<&'a str>,
    _marker: PhantomData<&'fmt ()>,
}

impl<'a> WrittenInner<'a, '_> {
    pub(crate) fn new(msg: &'a str) -> Self {
        Self {
            msg: Some(msg),
            _marker: PhantomData,
        }
    }

    pub(crate) fn prompt_with<R, W, F>(
        &mut self, mut read: R, mut write: W, fmt: &WrittenFmtRules<'_>, f: F,
    ) -> io::Result<String>
    where
        R: io::BufRead,
        W: io::Write,
        F: FnOnce(&mut R) -> io::Result<String>,
    {
        let fmt = fmt.expand();

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

    pub(crate) fn prompt<R, W>(
        &mut self, read: R, write: W, fmt: &WrittenFmtRules<'_>,
    ) -> io::Result<String>
    where
        R: io::BufRead,
        W: io::Write,
    {
        self.prompt_with(read, write, fmt, |read| {
            let mut s = String::new();
            read.read_line(&mut s)?;
            Ok(s)
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
        R: io::BufRead,
        W: io::Write,
    {
        let input = self.inner.prompt(read, write, fmt)?;
        match input.parse() {
            Ok(out) if !input.is_empty() => Ok(ControlFlow::Break(out)),
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::{
        format::{Expandable as _, FmtRule, rules::WrittenFmtRules},
        prelude::*,
    };

    #[test]
    fn normal_str_input() -> anyhow::Result<()> {
        let input = b"hello\n";
        let mut output = Vec::new();

        let res = crate::written::<String>("foobar")
            .prompt_with(BufReader::new(input.as_slice()), &mut output)?;
        assert_eq!(res, "hello");

        let default_fmt = WrittenFmtRules::default().expand();
        let expected_msg = format!(
            "{}foobar{}{}",
            default_fmt.msg_prefix,
            if default_fmt.break_line { "\n" } else { "" },
            default_fmt.input_prefix
        );
        assert_eq!(output.as_slice(), expected_msg.as_bytes());

        Ok(())
    }

    #[test]
    fn normal_int_input() -> anyhow::Result<()> {
        let input = b"34\n";
        let mut output = Vec::new();

        let res = crate::written::<i32>("foobi").prompt_with(input.as_slice(), &mut output)?;
        assert_eq!(res, 34);

        let default_fmt = WrittenFmtRules::default().expand();
        let expected_msg = format!(
            "{}foobi{}{}",
            default_fmt.msg_prefix,
            if default_fmt.break_line { "\n" } else { "" },
            default_fmt.input_prefix
        );
        assert_eq!(output.as_slice(), expected_msg.as_bytes());

        Ok(())
    }

    #[test]
    fn repeat_5_times_int_input() -> anyhow::Result<()> {
        let input = b"nop\nnop\nnop\nnop\n23\n";
        let mut output = Vec::new();

        let res = crate::written::<i32>("googa").prompt_with(input.as_slice(), &mut output)?;
        assert_eq!(res, 23);

        let default_fmt = WrittenFmtRules::default().expand();
        let expected_msg = format!(
            "{0}googa{1}{2}{3}{3}{3}{3}",
            default_fmt.msg_prefix,
            if default_fmt.break_line { "\n" } else { "" },
            default_fmt.input_prefix,
            if default_fmt.break_line {
                default_fmt.input_prefix.to_owned()
            } else {
                format!(
                    "{}googa{}",
                    default_fmt.msg_prefix, default_fmt.input_prefix
                )
            },
        );
        assert_eq!(output.as_slice(), expected_msg.as_bytes());

        Ok(())
    }

    #[test]
    fn fully_customized_fmt_with_good_input() -> anyhow::Result<()> {
        let input = b"hello\n";
        let mut output = Vec::new();

        let res = crate::written::<String>("booga")
            .fmt(
                crate::fmt()
                    .break_line(false)
                    .repeat_prompt(true)
                    .msg_prefix("* ")
                    .input_prefix(": "),
            )
            .prompt_with(input.as_slice(), &mut output)?;

        assert_eq!(res, "hello");
        assert_eq!(output.as_slice(), b"* booga: ");

        Ok(())
    }

    #[test]
    fn fully_customized_fmt_with_bad_input() -> anyhow::Result<()> {
        let input = b"hello\nhello\nhello\nhello\n2\n";
        let mut output = Vec::new();

        let res = crate::written::<i32>("booga")
            .fmt(
                crate::fmt()
                    .break_line(false)
                    .repeat_prompt(true)
                    .msg_prefix("* ")
                    .input_prefix(": "),
            )
            .prompt_with(input.as_slice(), &mut output)?;

        assert_eq!(res, 2);
        assert_eq!(
            output.as_slice(),
            b"* booga: * booga: * booga: * booga: * booga: "
        );

        Ok(())
    }
}
