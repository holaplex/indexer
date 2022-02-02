use chrono::{Duration, NaiveDateTime};

use crate::error::prelude::*;

/// Returns a tuple of `(ends_at, ended)`
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
