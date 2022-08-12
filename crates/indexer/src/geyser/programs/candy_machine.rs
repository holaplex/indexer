use anchor_lang_v0_21::{AccountDeserialize, AnchorDeserialize};
use arrayref::array_ref;
use mpl_candy_machine::{
    CandyMachine, CollectionPDA, ConfigLine, CONFIG_ARRAY_START, CONFIG_LINE_SIZE,
};
use mpl_token_metadata::state::{MAX_NAME_LENGTH, MAX_URI_LENGTH};

use super::{accounts::candy_machine, AccountUpdate, Client};
use crate::prelude::*;

const COLLECTION_PDA_SIZE: usize = 8 + 64;

/// parse config lines out of raw candy machine accounts data
/// lines that are not "available" are ignored (this would occur if the config lines are in the process
/// of being added, or failed to be added for whatever reason)
/// returns a vector containing tuples: (config_line, index, taken)
///
/// it is important that this not be called if the candy machine has hidden settings
pub fn parse_cm_config_lines(
    data: &Vec<u8>,
    items_available: usize,
) -> Vec<(ConfigLine, usize, bool)> {
    let config_line_start = CONFIG_ARRAY_START + 4;
    let available_bitmask_start = CONFIG_ARRAY_START + 4 + (items_available * CONFIG_LINE_SIZE) + 4;
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
    let bytes_needed_for_taken_bitmask =
        (items_available / 8) + if items_available % 8 == 0 { 0 } else { 1 };
    if taken_bitmask_start + bytes_needed_for_taken_bitmask >= data.len() {
        // TODO(will): Log warning
        return Vec::new();
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
            let config_line_byte_offset = config_line_start + (idx * CONFIG_LINE_SIZE);
            let config_line_data = array_ref![data, config_line_byte_offset, CONFIG_LINE_SIZE];
            let config_line_result = ConfigLine::deserialize(&mut config_line_data.as_slice());
            let taken_bitmask_byte_offset = idx / 8;
            let taken_bitmask_bit_offset = 7 - (idx % 8);
            let taken_bitmask_value = data[taken_bitmask_start + taken_bitmask_byte_offset];
            let taken = taken_bitmask_value & (1 << taken_bitmask_bit_offset) != 0;

            if let Ok(config_line) = config_line_result {
                config_lines.push((config_line, idx, taken));
            } else {
                // TODO(will): log some warning here that might alert us to a problem with this code?
            }
        }
    }

    return config_lines;
}

pub async fn process_collection_pda(client: &Client, update: AccountUpdate) -> Result<()> {
    let collection_pda: CollectionPDA = CollectionPDA::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize collection pda")?;

    candy_machine::process_collection_pda(client, update.key, collection_pda).await
}

pub async fn process_cm(client: &Client, update: AccountUpdate) -> Result<()> {
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize candy_machine")?;

    let items_available = usize::try_from(candy_machine.data.items_available);
    // TODO(will): log warning if conversion fails
    if candy_machine.data.hidden_settings.is_none() && items_available.is_ok() {
        // config lines are only used when hidden settings are not used
        let config_lines = parse_cm_config_lines(&update.data, items_available.unwrap());
        candy_machine::process(client, update.key, candy_machine, Some(config_lines)).await
    } else {
        candy_machine::process(client, update.key, candy_machine, None).await
    }
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        COLLECTION_PDA_SIZE => process_collection_pda(client, update).await,
        _ => process_cm(client, update).await,
    }
}
