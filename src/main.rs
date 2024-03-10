use iwant::Promptable as _;

fn main() {
    let (menu, hello) = iwant::selected([("first", 100), ("second", 200), ("more", 300)], ">> ")
        .then(iwant::written::<String>("hello", ": "))
        .prompt()
        .unwrap();

    println!("you selected {menu} then wrote {hello}");
}
