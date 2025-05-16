use ineed::prelude::*;

fn main() -> anyhow::Result<()> {
    let (name, age) = ineed::many_written("Name, age", ",")
        .until(|(_, age): &(String, i32)| *age > 5 && *age < 120)
        .prompt()?;
    println!("name={name}");
    println!("age={age}");

    Ok(())
}
