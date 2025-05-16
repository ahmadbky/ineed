//! You can ask for passwords, by enabling the "rpassword" feature.

use ineed::prelude::*;

fn main() -> anyhow::Result<()> {
    let password = ineed::password("Your new password")
        .then(ineed::password("Confirm"))
        .fmt(ineed::fmt().repeat_prompt(true))
        .until(|(a, b)| a == b)
        .map(|(password, _confirm)| password)
        .prompt()?;

    println!("We have safely registered your new password (btw it's {password})");

    Ok(())
}
