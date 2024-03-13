use iwant::Promptable as _;

fn main() {
    let ((nom, prénom, âge), _) =
        iwant::many_written::<(String, String, u8), 3>("votre nom et prénom et votre âge", " ")
            .then(iwant::written::<u8>("confirmer votre âge"))
            .until(|((_, _, âge), confirm_âge)| *âge == *confirm_âge)
            .prompt()
            .unwrap();

    println!("got {nom} and {prénom}, then {âge}")
}
