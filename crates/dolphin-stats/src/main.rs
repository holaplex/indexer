#![allow(clippy::pedantic, clippy::cargo, missing_docs)]

use std::sync::Arc;

use holaplex_indexer_dolphin_stats::*;
use indexer_core::{
    chrono::Duration,
    clap::Parser,
    db,
    db::{
        insert_into,
        models::{DolphinStats, DolphinStats1D},
        tables::dolphin_stats,
        update, Pool,
    },
    hash::HashSet,
    prelude::*,
    url::Url,
    util,
};
use tokio::sync::Semaphore;

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();
        debug!("{:#?}", opts);

        let Opts {
            dolphin_key,
            jobs,
            full,
            db,
        } = opts;

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let client = reqwest::Client::new();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let now = Utc::now();

        let url: Url = collections_endpoint().parse()?;
        let json = rt
            .block_on(async {
                client
                    .get(url.clone())
                    .header("Authorization", &dolphin_key)
                    .header("Content-Type", "application/json")
                    .send()
                    .await?
                    .json::<CollectionsResponse>()
                    .await
            })?
            .into_inner(|| &url)?;

        for Collection {
            symbol,
            supply,
            floor,
            listed,
            volume_all,
            ..
        } in &json
        {
            let run = || {
                supply.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection supply for {:?}", symbol))
                })?;
                floor.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection floor for {:?}", symbol))
                })?;
                listed.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection listed count for {:?}", symbol))
                })?;
                volume_all.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection volume for {:?}", symbol))
                })?;

                Result::<_>::Ok(())
            };

            match run() {
                Ok(()) => (),
                Err(e) => error!("{:?}", e),
            }
        }

        let symbols: Vec<_> = json
            .iter()
            .filter_map(|c| c.symbol.clone())
            .filter(|s| !s.is_empty())
            .collect();

        let time = Utc::now() - now;

        info!(
            "Collections fetch completed in {}",
            util::duration_hhmmssfff(time)
        );

        if cfg!(debug_assertions) {
            let symbol_set: HashSet<_> = symbols.iter().cloned().collect();

            assert_eq!(symbols.len(), symbol_set.len());
        }

        let now = Utc::now();

        let split_info = Stats {
            curr_1d: get_split(now, Duration::days(1)),
            curr_7d: get_split(now, Duration::days(7)),
            curr_30d: get_split(now, Duration::days(30)),
            last_1d: get_split(now, Duration::days(2)),
            last_7d: get_split(now, Duration::days(14)),
            last_30d: get_split(now, Duration::days(60)),
        };

        rt.block_on(async move {
            struct Shared {
                sem: Semaphore,
                dolphin_key: String,
                split_info: Stats<u64>,
                pool: Pool,
            }

            fn insert_full(
                sym: &str,
                Shared { pool, .. }: &Shared,
                floor: Stats<i64>,
                listed: Stats<i64>,
                volume: Stats<i64>,
            ) -> Result<()> {
                let conn = pool.get()?;
                let Stats {
                    curr_1d: floor_1d,
                    curr_7d: floor_7d,
                    curr_30d: floor_30d,
                    last_1d: last_floor_1d,
                    last_7d: last_floor_7d,
                    last_30d: last_floor_30d,
                } = floor;
                let Stats {
                    curr_1d: listed_1d,
                    curr_7d: listed_7d,
                    curr_30d: listed_30d,
                    last_1d: last_listed_1d,
                    last_7d: last_listed_7d,
                    last_30d: last_listed_30d,
                } = listed;
                let Stats {
                    curr_1d: volume_1d,
                    curr_7d: volume_7d,
                    curr_30d: volume_30d,
                    last_1d: last_volume_1d,
                    last_7d: last_volume_7d,
                    last_30d: last_volume_30d,
                } = volume;

                let row = DolphinStats {
                    collection_symbol: Borrowed(sym),
                    floor_1d,
                    floor_7d,
                    floor_30d,
                    listed_1d,
                    listed_7d,
                    listed_30d,
                    volume_1d,
                    volume_7d,
                    volume_30d,
                    last_floor_1d,
                    last_floor_7d,
                    last_floor_30d,
                    last_listed_1d,
                    last_listed_7d,
                    last_listed_30d,
                    last_volume_1d,
                    last_volume_7d,
                    last_volume_30d,
                    change_floor_1d: calc_percent_change(floor_1d, last_floor_1d),
                    change_floor_7d: calc_percent_change(floor_7d, last_floor_7d),
                    change_floor_30d: calc_percent_change(floor_30d, last_floor_30d),
                    change_volume_1d: calc_percent_change(volume_1d, last_volume_1d),
                    change_volume_7d: calc_percent_change(volume_7d, last_volume_7d),
                    change_volume_30d: calc_percent_change(volume_30d, last_volume_30d),
                    change_listed_1d: calc_percent_change(listed_1d, last_listed_1d),
                    change_listed_7d: calc_percent_change(listed_7d, last_listed_7d),
                    change_listed_30d: calc_percent_change(listed_30d, last_listed_30d),
                };

                insert_into(dolphin_stats::table)
                    .values(&row)
                    .on_conflict(dolphin_stats::collection_symbol)
                    .do_update()
                    .set(&row)
                    .execute(&conn)
                    .with_context(|| format!("Stats upsert for {:?} failed", sym))?;

                Result::<_>::Ok(())
            }

            fn insert_1d(
                sym: &str,
                Shared { pool, .. }: &Shared,
                floor: Stats<i64>,
                listed: Stats<i64>,
                volume: Stats<i64>,
            ) -> Result<()> {
                let conn = pool.get()?;
                let Stats {
                    curr_1d: floor_1d,
                    last_1d: last_floor_1d,
                    ..
                } = floor;
                let Stats {
                    curr_1d: listed_1d,
                    last_1d: last_listed_1d,
                    ..
                } = listed;
                let Stats {
                    curr_1d: volume_1d,
                    last_1d: last_volume_1d,
                    ..
                } = volume;

                let row = DolphinStats1D {
                    collection_symbol: Borrowed(sym),
                    floor_1d,
                    listed_1d,
                    volume_1d,
                    last_floor_1d,
                    last_listed_1d,
                    last_volume_1d,
                    change_floor_1d: calc_percent_change(floor_1d, last_floor_1d),
                    change_volume_1d: calc_percent_change(volume_1d, last_volume_1d),
                    change_listed_1d: calc_percent_change(listed_1d, last_listed_1d),
                };

                update(dolphin_stats::table.filter(dolphin_stats::collection_symbol.eq(sym)))
                    .set(row)
                    .execute(&conn)
                    .with_context(|| format!("One-day stats update for {:?} failed", sym))?;

                Result::<_>::Ok(())
            }

            let shared = Arc::new(Shared {
                sem: Semaphore::new(jobs),
                dolphin_key,
                split_info,
                pool,
            });

            let handles: Vec<_> = symbols
                .into_iter()
                .map(|s| {
                    let task = tokio::task::spawn({
                        let client = client.clone();
                        let s = s.clone();
                        let shared = Arc::clone(&shared);

                        async move {
                            let Shared {
                                ref sem,
                                ref dolphin_key,
                                ref split_info,
                                ..
                            } = *shared;

                            let permit = sem
                                .acquire()
                                .await
                                .context("Couldn't get semapore permit")?;
                            let url = market_stats_endpoint(
                                &s,
                                now - indexer_core::chrono::Duration::days(if full {
                                    60
                                } else {
                                    2
                                }),
                                now,
                            )?;
                            let json = client
                                .get(url.clone())
                                .header("Authorization", dolphin_key)
                                .header("Content-Type", "application/json")
                                .send()
                                .await
                                .with_context(|| format!("Request failed for {:?}", s))?
                                .json::<MarketStatsResponse>()
                                .await
                                .with_context(|| format!("Parsing JSON failed for {:?}", s))?
                                .into_inner(|| &url)?;

                            std::mem::drop(permit);

                            debug!("Completed {:?}", s);

                            #[allow(deprecated)]
                            let MarketStats {
                                floor_data,
                                listed_data,
                                volume_data,
                                holder_data,
                                volume_data_all: _,
                            } = json;

                            check_stats(&floor_data, "floor data", &s)?;
                            check_stats(&listed_data, "listed data", &s)?;
                            // check_stats(&volume_data, "volume data", &s)?;
                            check_stats(&holder_data, "holder data", &s)?;
                            check_stats(&volume_data, "delta volume data", &s)?;

                            let floor = split_stats(split_info, floor_data, |f| {
                                f.iter()
                                    .try_fold(None::<u64>, |s, (_, n)| {
                                        n.as_u64()
                                            .ok_or_else(|| anyhow!("Invalid number {:?}", n))
                                            .map(|n| Some(s.map_or(n, |s| s.min(n))))
                                    })?
                                    .unwrap_or(0)
                                    .try_into()
                                    .context("Value was too big to store")
                            })
                            .with_context(|| {
                                format!("Error while processing floor data for {:?}", s)
                            })?;

                            let listed = split_stats(split_info, listed_data, |f| {
                                f.iter()
                                    .try_fold(None::<u64>, |s, (_, n)| {
                                        n.as_u64()
                                            .ok_or_else(|| anyhow!("Invalid number {:?}", n))
                                            .map(|n| Some(s.map_or(n, |s| s.max(n))))
                                    })?
                                    .unwrap_or(0)
                                    .try_into()
                                    .context("Value was too big to store")
                            })
                            .with_context(|| {
                                format!("Error while processing listed data for {:?}", s)
                            })?;

                            let volume =
                                split_stats(split_info, volume_data, |f| {
                                    f.iter()
                                        .try_fold(None::<u64>, |s, (_, n)| {
                                            let n = n
                                                .as_u64()
                                                .ok_or_else(|| anyhow!("Invalid number {:?}", n))?;

                                            let next =
                                                s.map_or(Ok(n), |s| {
                                                    s.checked_add(n).ok_or_else(|| {
                                        anyhow!("Overflow when calculationg {:?} + {:?}", s, n)
                                    })
                                                })?;

                                            Result::<_>::Ok(Some(next))
                                        })?
                                        .unwrap_or(0)
                                        .try_into()
                                        .context("Value was too big to store")
                                })
                                .with_context(|| {
                                    format!("Error while processing volume data for {:?}", s)
                                })?;

                            tokio::task::spawn_blocking(move || {
                                if full {
                                    insert_full(&s, &*shared, floor, listed, volume)
                                } else {
                                    insert_1d(&s, &*shared, floor, listed, volume)
                                }
                            })
                            .await??;

                            Result::<_>::Ok(())
                        }
                    });

                    (s, task)
                })
                .collect();

            for (sym, handle) in handles {
                match handle
                    .await
                    .with_context(|| format!("Stats task for {:?} crashed", sym))
                    .and_then(|h| h.with_context(|| format!("Stats task for {:?} failed", sym)))
                {
                    Ok(()) => (),
                    Err(e) => error!("{:?}", e),
                }
            }
        });

        let time = Utc::now() - now;

        info!(
            "Stats fetch completed in {}",
            util::duration_hhmmssfff(time)
        );

        Ok(())
    });
}
