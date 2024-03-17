use ineed::prelude::*;

fn main() {
    let (password, _) = ineed::password("password")
        .then(ineed::password("confirm"))
        .until(|(a, b)| a == b)
        .fmt(ineed::fmt().repeat_prompt(true))
        .prompt()
        .unwrap();

    println!("got {password:?}");
}
