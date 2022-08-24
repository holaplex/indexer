use anchor_lang_v0_21::{AccountDeserialize, AnchorDeserialize};
use mpl_candy_machine::{
    CandyMachine, CollectionPDA, ConfigLine, CONFIG_ARRAY_START, CONFIG_LINE_SIZE,
};

use super::{accounts::candy_machine, AccountUpdate, Client};
use crate::prelude::*;

const COLLECTION_PDA_SIZE: usize = 8 + 64;

/// parse config lines out of raw candy machine accounts data
/// lines that are not "available" are ignored (this would occur if the config lines are in the process
/// of being added, or failed to be added for whatever reason)
/// returns a vector containing tuples: (`config_line`, index, taken)
///
/// it is important that this not be called if the candy machine has hidden settings
pub fn parse_cm_config_lines(
    data: &[u8],
    items_available: usize,
) -> Result<Vec<(ConfigLine, usize, bool)>> {
    const CONFIG_LINE_START: usize = CONFIG_ARRAY_START + 4;
    let available_bitmask_start = CONFIG_LINE_START + (items_available * CONFIG_LINE_SIZE) + 4;

    // NOTE(will): you would think that (items_available / 8) incorrectly computes the length of the
    // "available" bitmask. i.e. if there are 7 items available, this value would be 0.
    // however, this is how metaplex has coded it. It ends up working because there are 4 bytes of padding
    // left in between. It seems like probably this was intended to be a 32 bit integer indicating
    // the length of the "taken" bitmask, however, in practice, it is always all zeroes and the first byte
    // will get overwritten by the last byte of the "available" bitmask. When items_available is an exact
    // multiple of 8 everything is as you would expect, and there are just two extra bytes
    // at the end of the account (*face palm*)
    let taken_bitmask_start = available_bitmask_start + (items_available / 8) + 4;

    // Sanity check to make sure we aren't going to overflow data
    // This could occur if this function is called on a candy machine that uses hiddensettings instead of
    // config lines
    let bytes_needed_for_taken_bitmask = items_available / 8 + (items_available % 8).min(1);
    let total_expected_data_len = taken_bitmask_start + bytes_needed_for_taken_bitmask;

    if total_expected_data_len >= data.len() {
        bail!(
            "Config line bytes would overflow available data ({} vs {})",
            total_expected_data_len,
            data.len()
        );
    }

    // (config_line, index, taken)
    let mut config_lines: Vec<(ConfigLine, usize, bool)> = Vec::new();
    for idx in 0..items_available {
        let available_bitmask_byte_offset = idx / 8;
        let available_bitmask_bit_offset = 7 - (idx % 8);
        let available_bitmask_value = data[available_bitmask_start + available_bitmask_byte_offset];
        let available = available_bitmask_value & (1 << available_bitmask_bit_offset) != 0;

        // NOTE(will): if the config line is not available, we simply ignore it
        if available {
            let config_line_byte_offset = CONFIG_LINE_START + (idx * CONFIG_LINE_SIZE);
            let config_line = ConfigLine::deserialize(
                &mut &data[config_line_byte_offset..config_line_byte_offset + CONFIG_LINE_SIZE],
            )
            .with_context(|| format!("Failed to deserialize config line at index {}", idx))?;

            let taken_bitmask_byte_offset = idx / 8;
            let taken_bitmask_bit_offset = 7 - (idx % 8);
            let taken_bitmask_value = data[taken_bitmask_start + taken_bitmask_byte_offset];
            let taken = taken_bitmask_value & (1 << taken_bitmask_bit_offset) != 0;

            config_lines.push((config_line, idx, taken));
        }
    }

    Ok(config_lines)
}

pub async fn process_collection_pda(client: &Client, update: AccountUpdate) -> Result<()> {
    let collection_pda: CollectionPDA = CollectionPDA::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize collection pda")?;

    candy_machine::process_collection_pda(client, update.key, collection_pda).await
}

pub async fn process_cm(client: &Client, update: AccountUpdate) -> Result<()> {
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize candy_machine")?;

    let items_available = usize::try_from(candy_machine.data.items_available)
        .context("Failed to convert available item count")?;

    let lines = if candy_machine.data.hidden_settings.is_none() {
        Some(
            parse_cm_config_lines(&update.data, items_available)
                .context("Failed to parse candy machine lines")?,
        )
    } else {
        None
    };

    candy_machine::process(client, update.key, candy_machine, lines).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        COLLECTION_PDA_SIZE => process_collection_pda(client, update).await,
        _ => process_cm(client, update).await,
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, io::prelude::*, path::Path};

    use anchor_lang_v0_21::AccountDeserialize;
    use mpl_candy_machine::CandyMachine;

    use super::parse_cm_config_lines;
    use crate::prelude::*;

    fn load_account_dump(filename: impl AsRef<Path>) -> Result<Vec<u8>> {
        let mut path = env::current_dir().context("Failed to get working dir")?;
        path.extend(["tests", "data"]);
        path.push(filename);

        println!("Loading: {:?}", path);

        let mut f = fs::File::open(&path).with_context(|| format!("Failed to open {:?}", path))?;
        let mut buffer = vec![];
        f.read_to_end(&mut buffer)
            .with_context(|| format!("Failed to read {:?}", path))?;

        Ok(buffer)
    }

    #[test]
    fn test_can_deser_candy_machine() {
        let filenames = [
            "candy_machines/FhrVJL4xKNmAY53Bm5XJNqJwvBomDuDH7HGDdicgbkZY.dmp",
            "candy_machines/AoHidoffmkL4xURViNgbA4YyeDw82FAYUZfomL3X5BoU.dmp",
            "candy_machines/piA76RvvmCt7UWEmJSBVA6xMoXqwvEAELwJoqeHK6i3.dmp",
            "candy_machines/CiBuYi3W3aVQbMWcjvfKBpwjHS6fViuuxQdSUUqkjkn4.dmp",
            // This candy machine has hidden settings, but for some reason has allocated space
            // for all the config lines that would be needed, however they are all empty.
            // This results in returning a results array with zero as none of the config
            // lines are marked as available
            "candy_machines/ACDPaQ3uGy33KsBKiUH4azDX4q7Nxk3QwW3trALEdFmB.dmp",
        ];

        for filename in filenames {
            println!("Reading Candy Machine {:?}", filename);
            let data = load_account_dump(filename).unwrap();
            let cm = CandyMachine::try_deserialize(&mut data.as_slice()).unwrap();
            println!("Candy Machine: {}", filename);
            let avail = usize::try_from(cm.data.items_available).unwrap();
            let results = parse_cm_config_lines(&data, avail).unwrap();

            let available_count = results.len();
            let mut taken_count = 0;
            for (_, _, taken) in &results {
                if *taken {
                    taken_count += 1;
                }
            }

            if cm.data.hidden_settings.is_some() {
                assert_eq!(available_count, 0);
                assert_eq!(taken_count, 0);
            } else {
                assert_eq!(available_count, avail);
                assert_eq!(taken_count, cm.items_redeemed);
            }
        }
    }
}
