use anyhow::Context;
use docbot::prelude::*;

#[cfg(feature = "rabbitmq")]
mod rmq;
#[cfg(feature = "rpc")]
mod rpc;

#[derive(Docbot)]
enum BaseCommand {
    /// `rpc <subcommand...>`
    /// Make a request to the JSONRPC server
    ///
    /// # Arguments
    /// subcommand: The subcommand to run.
    #[cfg(feature = "rpc")]
    Rpc(#[docbot(subcommand)] rpc::RpcCommand),

    /// `mq <subcommand...>`
    /// Interact with a metaplex-indexer RabbitMQ instance
    ///
    /// # Arguments
    /// subcommand: The subcommand to run.
    #[cfg(feature = "rabbitmq")]
    Rmq(#[docbot(subcommand)] rmq::RmqCommand),
}

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

pub fn run() -> Result {
    let mut rl = rustyline::Editor::<()>::new();

    loop {
        use rustyline::error::ReadlineError;

        let line = match rl.readline("> ") {
            Ok(l) => l,
            Err(ReadlineError::Eof) => break Ok(()),
            Err(ReadlineError::Interrupted) => continue,
            Err(e) => {
                break Err(e).context("Failed to read user input");
            },
        };

        handle(line, |e| Ok(log::error!("{:?}", e)))
            .map_err(|e| log::error!("Command failed: {:?}", e))
            .ok();
    }
}

pub fn run_one(s: impl AsRef<str>) -> Result {
    handle(s, |e| Ok(log::error!("{:?}", e)))
}

fn handle(
    s: impl AsRef<str>,
    parse_error: impl FnOnce(docbot::CommandParseError) -> Result,
) -> Result {
    let cmd = match BaseCommand::parse(docbot::tokenize_str_simple(s.as_ref())) {
        Ok(c) => c,
        Err(e) => return parse_error(e),
    };

    match cmd {
        #[cfg(feature = "rpc")]
        BaseCommand::Rpc(r) => rpc::handle(r),
        #[cfg(feature = "rabbitmq")]
        BaseCommand::Rmq(r) => rmq::handle(r),
    }
}
