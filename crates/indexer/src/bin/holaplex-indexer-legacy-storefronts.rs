use indexer_core::{clap, prelude::*};

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(long, env)]
    arweave_url: String,
}

fn main() {
    holaplex_indexer::run(|args: Args, _params, db| async move {
        let Args { arweave_url } = args;

        holaplex_indexer::legacy_storefronts::run(
            &db,
            arweave_url.parse().context("Failed to parse Arweave URL")?,
        )
        .await
    })
}
