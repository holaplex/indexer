use indexer_core::{
    clap,
    clap::Parser,
    db,
    db::{tables::metadatas, update, Connection},
    prelude::*,
};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_client::RpcClient,
    rpc_request::RpcError,
};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(flatten)]
    db: db::ConnectArgs,
    #[clap(env)]
    solana_endpoint: String,
}

fn main() {
    indexer_core::run(|| {
        let Opts {
            db,
            solana_endpoint,
        } = Opts::parse();

        let client = RpcClient::new(&solana_endpoint);
        debug!("USING SOLANA ENDPOINT: {:?}", &solana_endpoint);

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let conn = pool.get()?;
        let eariliest_burn_slot = metadatas::table
            .filter(metadatas::burned_at.is_not_null())
            .order_by(metadatas::slot.asc())
            .select(metadatas::slot)
            .first::<Option<i64>>(&conn)
            .ok()
            .flatten()
            .context("could not load earliest slot")?;

        // TODO: Check for index on burned_at
        let total_count = metadatas::table
            .filter(metadatas::burned_at.is_null())
            .filter(metadatas::slot.lt(eariliest_burn_slot))
            .count()
            .get_result::<i64>(&conn)
            .context("could not get count")?;

        debug!("TOTAL UNBURNED COUNT: {:?}", total_count);

        let mut i = 0;
        let limit = 1000;
        while i < total_count {
            // slot less than the earliest burn slot;
            let addressess = metadatas::table
                .filter(metadatas::burned_at.is_null())
                .filter(metadatas::slot.lt(eariliest_burn_slot))
                .offset(i)
                .limit(limit)
                .order_by(metadatas::slot.asc())
                .select(metadatas::address)
                .load(&conn)
                .context("could not load metadatas")?;

            addressess.into_iter().try_for_each(|address: String| {
                let key: Pubkey = address.parse()?;
                let exists = account_exists(key, &client);
                if let Ok(false) = exists {
                    debug!("manually burning: {:?}", address);
                    remove_metadata(address, &conn, &client)?;
                }
                Result::<_>::Ok(())
            })?;

            i += limit;
        }

        Ok(())
    });
}

fn remove_metadata(metadata_address: String, conn: &Connection, client: &RpcClient) -> Result<()> {
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
