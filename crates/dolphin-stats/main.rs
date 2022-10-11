#![allow(clippy::pedantic, clippy::cargo)]

use indexer::prelude::*;
use indexer_core::{clap, clap::Parser, db};

#[derive(Debug, Parser)]
struct Opts {
    #[clap(flatten)]
    db: db::ConnectArgs,
}

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();
        debug!("{:#?}", opts);

        let Opts { db } = opts;

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let conn = pool.get()?;

        Ok(())
    });
}
