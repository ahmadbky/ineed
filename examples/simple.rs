use ineed::prelude::*;

#[derive(Debug)]
enum Level {
    Good,
    Medium,
    Bad,
}

fn main() -> anyhow::Result<()> {
    let username = ineed::written::<String>("Your username").prompt()?;
    let level = ineed::selected(
        "Your level",
        [
            ("Foo", Level::Good),
            ("Bar", Level::Medium),
            ("Foobar", Level::Bad),
        ],
    )
    .fmt(ineed::fmt().repeat_prompt(true))
    .prompt()?;

    println!("username={username}");
    println!("level={level:?}");

    Ok(())
}
