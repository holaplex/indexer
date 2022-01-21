use anyhow::Context;
use docbot::prelude::*;

#[cfg(feature = "rabbitmq")]
mod rmq;
#[cfg(feature = "rpc")]
mod rpc;

#[derive(Docbot)]
enum BaseCommand {
    /// `help [command...]`
    /// Print help for a command
    ///
    /// # Arguments
    /// command: The command to display info for.
    Help(#[docbot(path)] Option<BaseCommandPath>),

    /// `rpc <subcommand...>`
    /// Make a request to the JSONRPC server
    ///
    /// # Arguments
    /// subcommand: The subcommand to run.
    #[cfg(feature = "rpc")]
    #[docbot(subcommand)]
    Rpc(rpc::RpcCommand),

    /// `mq <subcommand...>`
    /// Interact with a metaplex-indexer RabbitMQ instance
    ///
    /// # Arguments
    /// subcommand: The subcommand to run.
    #[cfg(feature = "rabbitmq")]
    #[docbot(subcommand)]
    Rmq(rmq::RmqCommand),
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

        if let Ok(cmd) = parse(line) {
            handle(cmd)
                .map_err(|e| log::error!("Command failed: {:?}", e))
                .ok();
        }
    }
}

pub fn run_one(s: impl AsRef<str>) -> Result {
    handle(parse(s).unwrap_or_else(|()| std::process::exit(1)))
}

fn parse(s: impl AsRef<str>) -> Result<BaseCommand, ()> {
    BaseCommand::parse(docbot::tokenize_str_simple(s.as_ref())).map_err(|e| {
        log::trace!("{:?}", e);
        let err = docbot::SimpleFoldError
            .fold_command_parse(e)
            .unwrap_or_else(|e| format!("Failed to parse command: {:?}", e));

        if !err.is_empty() {
            eprintln!("{}", err);
        }
    })
}

fn handle(cmd: BaseCommand) -> Result {
    match cmd {
        BaseCommand::Help(c) => docbot::SimpleFoldHelp
            .fold_topic(BaseCommand::help(c))
            .context("Failed to format help")
            .map(|h| eprintln!("{}", h)),
        #[cfg(feature = "rpc")]
        BaseCommand::Rpc(r) => rpc::handle(r),
        #[cfg(feature = "rabbitmq")]
        BaseCommand::Rmq(r) => rmq::handle(r),
    }
}
