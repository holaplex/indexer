use clap::Parser;

mod repl;

#[derive(Parser)]
struct Opts {
    /// Run a single command and quit
    #[clap(short = 'c')]
    exec: Option<String>,
}

fn main() {
    env_logger::Builder::new()
        .filter_level(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .filter_module("rustyline", log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let Opts { exec } = Opts::parse();

    match exec.map_or_else(repl::run, repl::run_one) {
        Ok(()) => (),
        Err(e) => {
            log::error!("REPL exited with error: {:?}", e);
            std::process::exit(-1);
        },
    }
}
