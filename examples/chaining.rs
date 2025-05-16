//! This example shows how you can chain prompts with a single call.
//!
//! The chaining is designed so that if any user input is invalid during the chain, the prompt is
//! repeated from the beginning.

use ineed::prelude::*;

#[derive(Debug)]
enum Level {
    Good,
    Medium,
    Bad,
}

fn main() -> anyhow::Result<()> {
    let (username, level) = ineed::written::<String>("Your username")
        .then(ineed::selected(
            "Your level",
            [
                ("Good", Level::Good),
                ("Medium", Level::Medium),
                ("Bad", Level::Bad),
            ],
        ))
        .prompt()?;

    println!("username={username}");
    println!("level={level:?}");

    Ok(())
}
