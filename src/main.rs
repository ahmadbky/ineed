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
            .fmt(
                iwant::fmt()
                    .break_line(false)
                    .input_prefix(": ")
                    .list_surrounds("<", ">")
                    .list_msg_pos(iwant::format::Position::Bottom),
            ),
        )
        .fmt(iwant::fmt().input_prefix(">> ").repeat_prompt(true))
        .prompt()
        .unwrap();

    println!("got {author} and {license:?}");
}
