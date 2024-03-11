use iwant::Promptable as _;

fn main() {
    let (nom, prénom, age) =
        iwant::many_written::<(String, String), 2>("votre nom et prénom", ": ", " ")
            .then(iwant::written::<u8>("age", ": "))
            .prompt()
            .unwrap();

    println!("got {nom} and {prénom}, then {age}")
}
