//! This example shows how you can customize the format of the prompts.
//!
//! Each promptable type accepts its own set of format rules. For example for
//! [written](ineed::written) inputs, you can provide a custom prefix for the user input (e.g. "> ").
//! However, you can't provide a custom position for the message, as this rule is specific for
//! [selectable](ineed::selected) prompts.
//!
//! You can find these specifications in the documentation of the various
//! [rule types](ineed::format::rules).

use ineed::prelude::*;

fn main() -> anyhow::Result<()> {
    let age = ineed::written::<u8>("Your age")
        .fmt(
            ineed::fmt()
                .break_line(false)
                .input_prefix(": ")
                .repeat_prompt(true),
        )
        .prompt()?;

    println!("Got: {age}");

    Ok(())
}
