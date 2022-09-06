#![allow(clippy::pedantic, clippy::cargo)]

use indexer::prelude::*;
use indexer_core::{
    clap,
    clap::Parser,
    db,
    db::{
        tables::{current_metadata_owners, metadatas},
        update,
    },
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

        let total_count = metadatas::table
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .filter(metadatas::burned_at.is_null())
            .count()
            .get_result::<i64>(&conn)
            .context("Could not get count")?;

        debug!("Total unburned count: {}", total_count);

        const LIMIT: i64 = 10000;

        for i in (0..total_count).step_by(LIMIT as usize) {
            let addresses = metadatas::table
                .inner_join(
                    current_metadata_owners::table
                        .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
                )
                .filter(metadatas::burned_at.is_null())
                .offset(i)
                .limit(LIMIT)
                .order_by(metadatas::slot.asc())
                .select((
                    current_metadata_owners::token_account_address,
                    metadatas::address,
                ))
                .load(&conn)
                .context("Could not load metadatas")?;

            addresses
                .into_iter()
                .try_for_each(|(token_address, metadata): (String, String)| {
                    let key: Pubkey = token_address.parse()?;
                    let exists = account_exists(key, &client);

                    if let Ok(false) = exists {
                        debug!("Manually burning: {:?}", metadata);
                        remove_metadata(metadata, &conn)?;
                    }

                    Result::<_>::Ok(())
                })?;
        }

        Ok(())
    });
}

fn remove_metadata(metadata_address: String, conn: &db::Connection) -> Result<()> {
    let now = Local::now().naive_utc();

    update(metadatas::table.filter(metadatas::address.eq(metadata_address)))
        .set(metadatas::burned_at.eq(now))
        .execute(conn)
        .context("couldnt set burned_at")?;

    Ok(())
}

fn account_exists(token_account: Pubkey, client: &RpcClient) -> Result<bool> {
    let result = client.get_account(&token_account);
    match result {
        Err(ClientError {
            request: _,
            kind: ClientErrorKind::RpcError(RpcError::ForUser(_)),
        }) => Ok(false),
        Err(e) => bail!("RPC ERROR: {:?}", e),
        Ok(_) => Ok(true),
    }
}
