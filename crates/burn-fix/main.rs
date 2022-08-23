#![allow(clippy::pedantic, clippy::cargo)]

use indexer::prelude::*;
use indexer_core::{
    clap,
    clap::Parser,
    db,
    db::{tables::metadatas, update},
};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_client::RpcClient,
    rpc_request::RpcError,
};

#[derive(Debug, Parser)]
struct Opts {
    /// Solana RPC endpoint
    #[clap(long, env)]
    solana_endpoint: String,

    #[clap(flatten)]
    db: db::ConnectArgs,
}

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();
        debug!("{:#?}", opts);

        let Opts {
            solana_endpoint,
            db,
        } = opts;

        let client = RpcClient::new(&solana_endpoint);

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let conn = pool.get()?;
        let earliest_burn_slot = metadatas::table
            .filter(metadatas::burned_at.is_not_null())
            .order_by(metadatas::slot.asc())
            .select(metadatas::slot)
            .first::<Option<i64>>(&conn)
            .ok()
            .flatten()
            .context("Could not load earliest slot")?;

        // TODO: Check for index on burned_at
        let total_count = metadatas::table
            .filter(metadatas::burned_at.is_null())
            .filter(metadatas::slot.lt(earliest_burn_slot))
            .count()
            .get_result::<i64>(&conn)
            .context("Could not get count")?;

        debug!("Total unburned count: {}", total_count);

        const LIMIT: i64 = 1000;

        for i in (0..total_count).step_by(LIMIT as usize) {
            // slot less than the earliest burn slot;
            let addressess = metadatas::table
                .filter(metadatas::burned_at.is_null())
                .filter(metadatas::slot.lt(earliest_burn_slot))
                .offset(i)
                .limit(LIMIT)
                .order_by(metadatas::slot.asc())
                .select(metadatas::address)
                .load(&conn)
                .context("Could not load metadatas")?;

            addressess.into_iter().try_for_each(|address: String| {
                let key: Pubkey = address.parse()?;
                let exists = account_exists(key, &client);

                if let Ok(false) = exists {
                    debug!("Manually burning: {:?}", address);
                    remove_metadata(address, &conn, &client)?;
                }

                Result::<_>::Ok(())
            })?;
        }

        Ok(())
    });
}

fn remove_metadata(
    metadata_address: String,
    conn: &db::Connection,
    client: &RpcClient,
) -> Result<()> {
    let now = Local::now().naive_utc();
    let slot = i64::try_from(client.get_slot().context("failed to get slot")?)?;

    update(metadatas::table.filter(metadatas::address.eq(metadata_address)))
        .set((metadatas::burned_at.eq(now), metadatas::slot.eq(slot)))
        .execute(conn)
        .context("couldnt set burned_at")?;

    Ok(())
}

fn account_exists(metadata_address: Pubkey, client: &RpcClient) -> Result<bool> {
    let result = client.get_account(&metadata_address);
    match result {
        Err(ClientError {
            request: _,
            kind: ClientErrorKind::RpcError(RpcError::ForUser(_)),
        }) => Ok(false),
        Err(e) => bail!("RPC ERROR: {:?}", e),
        Ok(_) => Ok(true),
    }
}
