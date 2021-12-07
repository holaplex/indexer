use std::{
    borrow::Cow,
    io::{prelude::*, stdin, stdout},
};

use bs58::{decode, encode};
use clap::Parser;

#[derive(Parser)]
struct Opts {
    /// Disable stripping a final `\n` off of STDIN
    #[clap(long, short = 'S')]
    no_strip: bool,

    /// Use hexadecimal instead of raw binary
    #[clap(long, short = 'x')]
    hex: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Encode,
    Decode,
}

fn main() {
    let Opts {
        no_strip,
        hex,
        command,
    } = Opts::parse();

    let mut contents = Vec::new();
    stdin().read_to_end(&mut contents).unwrap();

    let contents = if no_strip {
        &contents
    } else {
        std::str::from_utf8(&contents)
            .ok()
            .and_then(|s| s.rsplit_once('\n').map(|(l, _)| l.as_bytes()))
            .unwrap_or(&contents)
    };

    match command {
        Command::Encode => {
            stdout()
                .write_all(
                    encode(if hex {
                        Cow::Owned(hex::decode(contents).unwrap())
                    } else {
                        Cow::Borrowed(contents)
                    })
                    .into_string()
                    .as_bytes(),
                )
                .unwrap();
        },
        Command::Decode => {
            let bin = &decode(contents).into_vec().unwrap();

            stdout()
                .write_all(
                    if hex {
                        Cow::Owned(hex::encode(bin).into_bytes())
                    } else {
                        Cow::Borrowed(bin)
                    }
                    .as_ref(),
                )
                .unwrap();
        },
    }
}
