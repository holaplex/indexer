use anchor_lang_v0_21::{AccountDeserialize, AnchorDeserialize};
use arrayref::array_ref;
use mpl_candy_machine::{
    CandyMachine, CollectionPDA, ConfigLine, CONFIG_ARRAY_START, CONFIG_LINE_SIZE,
};
use mpl_token_metadata::state::{MAX_NAME_LENGTH, MAX_URI_LENGTH};

use super::{accounts::candy_machine, AccountUpdate, Client};
use crate::prelude::*;

const COLLECTION_PDA_SIZE: usize = 8 + 64;

/// config lines are stored in the candy machine account data, but are not deserialized
/// into the candy machine struct, this function will unparse them
pub fn parse_cm_config_lines(
    data: &Vec<u8>,
    items_available: usize,
) -> Result<Vec<(ConfigLine, bool, bool)>> {
    // result is config line, available,
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

pub fn check_for_bitmask_containing_n_ones(data: &Vec<u8>, items_available: usize, n: u64) {
    let mut current_index: usize = CONFIG_ARRAY_START + 4 + (items_available * CONFIG_LINE_SIZE);
    let num_bytes_needed_for_bitmask =
        items_available / 8 + if items_available % 8 != 0 { 1 } else { 0 };

    while current_index < (data.len() - num_bytes_needed_for_bitmask) {
        let mut bitmask_strings: Vec<String> = Vec::new();

        for i in 0..num_bytes_needed_for_bitmask as usize {
            bitmask_strings.push(format!("{:#010b}", data[current_index + i]).replace("0b", ""));
        }

        let ones_count = bitmask_strings.join("").matches("1").count();
        if ones_count as u64 == n {
            println!(
                "Found ones count equal to {} at byte offset: {}",
                ones_count, current_index
            );
        }

        current_index += 1;
    }
}

// const MAX_NAME_LENGTH: usize = 32;
// const MAX_URI_LENGTH: usize = 200;
// pub const CONFIG_LINE_SIZE: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
pub fn dump_cm_config_lines(data: &Vec<u8>, items_available: usize) {
    let mut current_index: usize = CONFIG_ARRAY_START;
    let current_count = u32::from_le_bytes(*array_ref![data, current_index, 4]);
    println!("current_count: {}", current_count);
    current_index += 4;

    for i in 0..current_count {
        let config_line_name_len = u32::from_le_bytes(*array_ref![data, current_index, 4]);
        current_index += 4;
        let config_line_name_bytes = array_ref![data, current_index, MAX_NAME_LENGTH];
        current_index += MAX_NAME_LENGTH;
        let config_line_uri_len = u32::from_le_bytes(*array_ref![data, current_index, 4]);
        current_index += 4;
        let config_line_uri_bytes = array_ref![data, current_index, MAX_URI_LENGTH];
        current_index += MAX_URI_LENGTH;

        let name_str = String::from_utf8(config_line_name_bytes.to_vec()).unwrap();
        let uri_str = String::from_utf8(config_line_uri_bytes.to_vec()).unwrap();

        let print_config_lines = false;
        if print_config_lines {
            println!(
                "Line: {}: [{}]:[{:?}]:[{}]:[{:?}]",
                i,
                config_line_name_len,
                name_str.trim_matches(char::from(0)),
                config_line_uri_len,
                uri_str.trim_matches(char::from(0))
            )
        }
    }

    let bitmask_vec_len = u32::from_le_bytes(*array_ref![data, current_index, 4]) as usize;
    current_index += 4;
    println!("bitmask_vec_len: {}", bitmask_vec_len);

    let mut bitmask_strings: Vec<String> = Vec::new();
    // NOTE(will): for some reason the field thats supposed to store the length of the bitmask vector
    // is off by 1, thus, the + 1 here
    for i in 0..(bitmask_vec_len + 1) {
        bitmask_strings.push(format!("{:#010b}", data[current_index]));
        current_index += 1;
    }
    println!("bitmaks: {:?}", bitmask_strings.join("|"));

    // // no idea wtf this represents
    // let some_number = u32::from_le_bytes(*array_ref![data, current_index, 4]) as usize;
    // current_index += 4;
    // println!("some_number: {}", some_number);

    println!("assumed start of second bitmask: {}", current_index);
    let mut taken_bitmask_strings: Vec<String> = Vec::new();
    // NOTE(will): for some reason the field thats supposed to store the length of the bitmask vector
    // is off by 1, thus, the + 1 here
    for i in 0..(bitmask_vec_len + 1 + 4) {
        taken_bitmask_strings.push(format!("{:#010b}", data[current_index]));
        current_index += 1;
    }
    println!("taken: {:?}", taken_bitmask_strings.join("|"));

    let mut leftover_bytes: Vec<String> = Vec::new();
    while current_index < data.len() {
        leftover_bytes.push(format!("{:010b}", data[current_index]));
        current_index += 1
    }

    println!(
        "leftover_bytes (shoudnt be any): {:?}",
        leftover_bytes.join("|")
    );

    println!(
        "first bitmask bit counts: 1's:{} 0's:{}",
        bitmask_strings
            .join("|")
            .replace("0b", "")
            .matches("1")
            .count(),
        bitmask_strings
            .join("|")
            .replace("0b", "")
            .matches("0")
            .count(),
    );

    println!(
        "second bitmask bit counts: 1's:{} 0's:{}",
        taken_bitmask_strings
            .join("")
            .replace("0b", "")
            .matches("1")
            .count(),
        taken_bitmask_strings
            .join("")
            .replace("0b", "")
            .matches("0")
            .count(),
    );
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

    candy_machine::process(client, update.key, candy_machine).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        COLLECTION_PDA_SIZE => process_collection_pda(client, update).await,
        CONFIG_LINE_SIZE => process_config_line(client, update).await,
        _ => process_cm(client, update).await,
    }
}
