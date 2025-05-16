//! This example shows how you can customize the format of a whole chaining prompt.
//!
//! The accepted format rules for the chain is the intersection of the accepted format rules
//! of every prompt it contains. For example, if we're prompting the chain C = A + B, and the prompt
//! A accepts the rules [msg_prefix, input_prefix], and B accepts the rules [msg_prefix, repeat_prompt],
//! C will accept the rules [msg_prefix].
//!
//! You can still customize each prompt more specifically. For example, by doing
//! `ineed::written(...).fmt(...).then(...)` instead of `ineed::written(...).then(...).fmt(...)`.
//! In such case, if you provide a custom format for a specific promptable AND for the chain, with
//! conflicting rules, then the rules used are those defined for the specific promptable,
//! as they have more precedence.

use ineed::prelude::*;

#[derive(Debug)]
enum Level {
    Good,
    Medium,
    Bad,
}

fn main() -> anyhow::Result<()> {
    let (name, level) = ineed::written::<String>("Your name")
        .then(
            ineed::selected(
                "Your age",
                [
                    ("Good", Level::Good),
                    ("Medium", Level::Medium),
                    ("Bad", Level::Bad),
                ],
            )
            .fmt(ineed::fmt().input_prefix("> ").list_surrounds("<", "> ")),
        )
        // Notice that we can't provide `.list_surrounds(...)` here, as the written promptable
        // doesn't accept it. We provide it specifically for the selected prompt.
        // Also, we already provided `input_prefix` for the selected prompt. For the latter, the one
        // defined specifically for it will be displayed instead of this prefix.
        .fmt(ineed::fmt().input_prefix(">> ").msg_prefix("-> "))
        .prompt()?;

    println!("name={name}");
    println!("level={level:?}");

    Ok(())
}
