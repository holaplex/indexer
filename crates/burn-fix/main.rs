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

    #[clap(env)]
    batch_size: i64,
}

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();
        debug!("{:#?}", opts);

        let Opts {
            solana_endpoint,
            db,
            batch_size,
        } = opts;

        let client = RpcClient::new(&solana_endpoint);

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let conn = pool.get()?;

        let total_count = metadatas::table
            .filter(metadatas::burned_at.is_null())
            .count()
            .get_result::<i64>(&conn)
            .context("Could not get count")?;

        debug!("Total unburned count: {}", total_count);

        for i in (0..total_count).step_by(batch_size as usize) {
            let addresses = metadatas::table
                .filter(metadatas::burned_at.is_null())
                .offset(i)
                .limit(batch_size)
                .order_by(metadatas::slot.asc())
                .select(metadatas::mint_address)
                .load(&conn)
                .context("Could not load metadatas")?;

            addresses.into_iter().try_for_each(|mint_address: String| {
                let key: Pubkey = mint_address.parse()?;
                let exists = account_exists(key, &client);

                if let Ok(false) = exists {
                    debug!("Manually burning: {:?}", mint_address);
                    remove_metadata(mint_address, &conn)?;
                }

                Result::<_>::Ok(())
            })?;
        }

        Ok(())
    });
}

fn remove_metadata(mint_address: String, conn: &db::Connection) -> Result<()> {
    let now = Local::now().naive_utc();

    update(metadatas::table.filter(metadatas::mint_address.eq(mint_address)))
        .set(metadatas::burned_at.eq(now))
        .execute(conn)
        .context("couldnt set burned_at")?;

    Ok(())
}

fn account_exists(token_account: Pubkey, client: &RpcClient) -> Result<bool> {
    let result = client.get_token_supply(&token_account);
    match result {
        Ok(token_amount) => {
            if token_amount.amount == "0" {
                Ok(false)
            } else {
                Ok(true)
            }
        },
        Err(ClientError {
            request: _,
            kind: ClientErrorKind::RpcError(RpcError::RpcResponseError { code, .. }),
        }) if code == -32602 => Ok(false),
        Err(e) => bail!("RPC ERROR: {:?}", e),
    }
}
