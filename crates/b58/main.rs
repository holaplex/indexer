use std::io::{prelude::*, stdin, stdout};

use bs58::{decode, encode};
use clap::Parser;

#[derive(Parser)]
enum Command {
    Encode,
    Decode,
}

fn main() {
    let cmd = Command::parse();

    let mut contents = Vec::new();
    stdin().read_to_end(&mut contents).unwrap();

    match cmd {
        Command::Encode => {
            stdout()
                .write_all(encode(contents).into_string().as_bytes())
                .unwrap();
        },
        Command::Decode => {
            stdout()
                .write_all(&decode(contents).into_vec().unwrap())
                .unwrap();
        },
    }
}
