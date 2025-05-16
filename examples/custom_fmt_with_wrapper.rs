//! This module shows how you can customize the prompt format many times for a single prompt.
//!
//! In such case, if any format rule conflicts, the precedence goes to the one defined closest
//! to the promptable in question.

use ineed::prelude::*;

fn main() -> anyhow::Result<()> {
    // Here, the input prefix will be ">> ", as it's the one defined closest to the
    // written promptable.
    let age = ineed::written::<u8>("Your age")
        .fmt(ineed::fmt().input_prefix(">> ").msg_prefix("-> "))
        .until(|age| *age > 3 && *age < 120)
        .fmt(ineed::fmt().input_prefix("=> ").repeat_prompt(true))
        .prompt()?;

    println!("You are {age}");

    Ok(())
}
