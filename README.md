# ineed

[![](https://img.shields.io/crates/v/ineed?style=flat-square)](https://crates.io/crates/ineed) ![](https://img.shields.io/docsrs/ineed?style=flat-square)

`ineed` is a lightweight CLI prompting Rust crate.

It provides utility traits and types to prompt values in a more convenient way, and allows you to customize the style of the prompts.

## Usage

First, add the crate to your dependency:
```
cargo add ineed
```

Then, add this line at the beginning of your code:
```
use ineed::prelude::*;
```

You are ready to use this crate. If you need a written value of any type, let's say `u8`, you can just ask for it:
```rust
let age = ineed::written::<u8>("How old are you?").prompt().unwrap();
```

`ineed` will manage the prompt format for you, and the input checking.

This example prints something similar to this:

```txt
- How old are you?
>
```

## Features

You want to customize the prompt? Use the `.fmt(...)` method like this:

```rust
let age = ineed::written::<u8>("How old are you?")
    .fmt(ineed::fmt().input_prefix(">> "))
    .prompt()
    .unwrap();
println!("You are {age}!");
```

And the last line will be `>> ` instead of `> ` from the previous example.

You can also ask for a selectable value:

```rust
enum LicenseType {
    MIT,
    GPL,
    BSD,
}

let license = ineed::selected("The license type", [
    ("MIT", LicenseType::MIT),
    ("GPL", LicenseType::GPL),
    ("BSD", LicenseType::BSD),
])
    .prompt()
    .unwrap();
```

Or compose the prompts into one binding:

```rust
#[derive(Debug)]
enum Age {
    Minor,
    LegalAge,
    Unknown,
}

let (name, age) = ineed::written::<String>("Your name")
    .then(
        ineed::written::<u8>("How old are you?")
            .max_tries(3)
            .map(|age| match age {
                Ok(..18) => Age::Minor,
                Ok(_) => Age::LegalAge,
                Err(_exceeded) => Age::Unknown,
            })
    )
    .prompt()
    .unwrap();

println!("Your name is {name} and your age status is {age:?}");
```

There are many other promptable types, which are listed in the [crate documentation](https://docs.rs/ineed).

You can also prompt for password. For this, you must add the `rpassword` feature:
```
cargo add ineed -F rpassword
```

which will give you access to the [`ineed::password`](https://docs.rs/ineed/latest/ineed/fn.password.html) promptable.

You can find more examples in the [/examples] folder.
