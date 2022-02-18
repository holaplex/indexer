//! Various indexer support utilities

use chrono::{Duration, NaiveDateTime};

use crate::error::prelude::*;
///
/// Format a [`chrono::Duration`] in HH:MM:SS.FFF format
#[must_use]
pub fn duration_hhmmssfff(duration: chrono::Duration) -> String {
    use std::fmt::Write;

    let mut out = String::new();

    let h = duration.num_hours();
    if h > 0 {
        write!(out, "{:02}:", h).unwrap();
    }

    write!(
        out,
        "{:02}:{:02}.{:03}",
        duration.num_minutes().rem_euclid(60),
        duration.num_seconds().rem_euclid(60),
        duration.num_milliseconds().rem_euclid(1000)
    )
    .unwrap();

    out
}

/// Returns a tuple of `(ends_at, ended)`
///
/// # Errors
/// This function fails of the end time cannot be safely computed.
pub fn get_end_info(
    ends_at: Option<NaiveDateTime>,
    gap_time: Option<Duration>,
    last_bid_time: Option<NaiveDateTime>,
    now: NaiveDateTime,
) -> Result<(Option<NaiveDateTime>, bool)> {
    // Based on AuctionData::ended
    let ends_at = match (ends_at, gap_time, last_bid_time) {
        (Some(end), Some(gap), Some(last)) => Some(
            end.max(
                last.checked_add_signed(gap)
                    .ok_or_else(|| anyhow!("Failed to adjust auction end by gap time"))?,
            ),
        ),
        (end, ..) => end,
    };

    let ended = ends_at.map_or(false, |e| now > e);

    Ok((ends_at, ended))
}
