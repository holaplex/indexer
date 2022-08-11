use anchor_lang_v0_21::{AccountDeserialize, AnchorDeserialize};
use arrayref::array_ref;
use mpl_candy_machine::{
    CandyMachine, CollectionPDA, ConfigLine, CONFIG_ARRAY_START, CONFIG_LINE_SIZE,
};

use super::{accounts::candy_machine, AccountUpdate, Client};
use crate::prelude::*;

const COLLECTION_PDA_SIZE: usize = 8 + 64;

/// config lines are stored in the candy machine account data, but are not deserialized
/// into the candy machine struct. This utility function unfucks metaplex's weird custom ser/deser
/// methodology
pub fn parse_cm_config_lines(
    data: &Vec<u8>,
    items_available: usize,
) -> Result<Vec<Option<ConfigLine>>> {
    // get the number of bytes in the occupancy bitmask
    let occupancy_bitmask_vec_length_offset = CONFIG_ARRAY_START  // other candymachine data
        + 4                                                       // u32 - number of config lines occupied
        + (items_available * CONFIG_LINE_SIZE); // space for config lines
    // + 4
    // + (items_available / 8)
    // + 4;

    let bitmask_vec_len =
        u32::from_le_bytes(*array_ref![data, occupancy_bitmask_vec_length_offset, 4]) as usize;

    // get other useful offsets
    let current_count = u32::from_le_bytes(*array_ref![data, CONFIG_ARRAY_START, 4]) as usize;
    let occupancy_bitmask_vec_offset = occupancy_bitmask_vec_length_offset + 4;
    // + (items_available / 8) + 4;
    let config_lines_offset = CONFIG_ARRAY_START + 4;

    println!("Current Count: {}", current_count);
    println!("Bitmask Vec Len: {}", bitmask_vec_len + 1);

    let mut results: Vec<Option<ConfigLine>> = Vec::new();

    // iterate over BITS in the bitmask here
    // NOTE(will): for
    for i in 0..((bitmask_vec_len + 1) * 8) {
        let byte_offset = i / 8;
        let bit_offset = 7 - (i % 8);
        let bitmask_value = data[occupancy_bitmask_vec_offset + byte_offset];
        let occupied = bitmask_value & (1 << bit_offset) != 0;
        println!(
            "byte: {:?} bit {:?} in byte: {:#010b} is occupied? {:?}",
            byte_offset, bit_offset, bitmask_value, occupied
        );
        if occupied {
            let config_line_offset = config_lines_offset + (i * CONFIG_LINE_SIZE);
            let config_line_data = array_ref![data, config_line_offset, CONFIG_LINE_SIZE];
            let config_line: ConfigLine =
                ConfigLine::deserialize(&mut config_line_data.as_slice())?;
            results.push(Some(config_line));
        } else {
            results.push(None);
        }
    }

    return Ok(results);
}

pub async fn process_collection_pda(client: &Client, update: AccountUpdate) -> Result<()> {
    let collection_pda: CollectionPDA = CollectionPDA::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize collection pda")?;

    candy_machine::process_collection_pda(client, update.key, collection_pda).await
}

pub async fn process_config_line(client: &Client, update: AccountUpdate) -> Result<()> {
    let config_line: ConfigLine = ConfigLine::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize config line")?;

    candy_machine::process_config_line(client, update.key, config_line).await
}

pub async fn process_cm(client: &Client, update: AccountUpdate) -> Result<()> {
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize candy_machine")?;

    // let num_config_lines = get_config_count(&mut update.data.as_slice());

    // NOTE(will): need to delete config lines that aren't present but have the same CM address

    candy_machine::process(client, update.key, candy_machine).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        COLLECTION_PDA_SIZE => process_collection_pda(client, update).await,
        CONFIG_LINE_SIZE => process_config_line(client, update).await,
        _ => process_cm(client, update).await,
    }
}
