use ineed::prelude::*;

fn main() {
    let res = ineed::bool("msg").prompt().unwrap();
    println!("got {res}");
}
