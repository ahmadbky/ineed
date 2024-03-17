use iwant::prelude::*;

#[derive(Debug)]
enum License {
    Mit,
    Gpl,
    Bsd,
    Apache,
}

fn main() {
    let (author, license) = iwant::written::<String>("author")
        .then(
            iwant::selected(
                "choose the license",
                [
                    ("MIT", License::Mit),
                    ("GPL", License::Gpl),
                    ("BSD", License::Bsd),
                    ("Apache", License::Apache),
                ],
            )
            .fmt(iwant::fmt().list_surrounds("<", ">")),
        )
        .fmt(iwant::fmt().input_prefix(">> ").repeat_prompt(true))
        .prompt()
        .unwrap();

    println!("got {author} and {license:?}");
}
