use holaplex_indexer_core::{clap, clap::Parser, db, prelude::*};

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
struct Opts {
    #[command(flatten)]
    db: db::ConnectArgs,
}

fn main() {
    holaplex_indexer_core::run(|| {
        let Opts { db } = Opts::parse();

        let db::ConnectResult {
            pool: _,
            ty: _,
            migrated,
        } = db::connect(db, db::ConnectMode::Write { migrate: true })?;

        if !migrated {
            bail!("Database was read-only, no migrations were run");
        }

        Ok(())
    });
}
