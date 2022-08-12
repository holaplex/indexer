use core::convert::Into;
use std::{
    env::current_dir,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anchor_lang_v0_21::{AccountDeserialize, AnchorDeserialize};
use holaplex_indexer::geyser::programs::candy_machine::parse_cm_config_lines;
use mpl_candy_machine::{CandyMachine, CollectionPDA, ConfigLine, CONFIG_LINE_SIZE};
use solana_program::example_mocks::solana_sdk;

fn get_file_as_byte_vec<P: AsRef<Path>>(filename: P) -> Vec<u8> {
    let mut f = fs::File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
fn get_data_dir() -> std::path::PathBuf {
    let mut data_dir = std::env::current_dir().unwrap();
    data_dir.push("tests");
    data_dir.push("data");
    data_dir.to_owned()
}

fn load_account_dump<P: AsRef<Path>>(filename: P) -> Vec<u8> {
    let data_dir = get_data_dir();
    let full_path = data_dir.join(filename);
    println!("Loading: {:?}", full_path);

    return get_file_as_byte_vec(full_path);
}

#[tokio::test]
#[cfg_attr(not(feature = "test-internal"), ignore)]
#[cfg_attr(not(feature = "geyser"), ignore)]
async fn test_can_deser_candy_machine() {
    let filenames = [
        "FhrVJL4xKNmAY53Bm5XJNqJwvBomDuDH7HGDdicgbkZY.dmp",
        "AoHidoffmkL4xURViNgbA4YyeDw82FAYUZfomL3X5BoU.dmp",
        "piA76RvvmCt7UWEmJSBVA6xMoXqwvEAELwJoqeHK6i3.dmp",
        "FsqgJVhKydM73N6TRubx1JUMhbccmwnQhZHzZQKiuJ1.dmp",
        "CMyRRHXkL4uJ78pzXq8wFDD3wv8jQBxuW27p2xzG3UBV.dmp",
        "2drzAbbL7AZPASmXQp7Qk5jdr4GXXbbLPgMhUhn5j5xd.dmp",
        "CiBuYi3W3aVQbMWcjvfKBpwjHS6fViuuxQdSUUqkjkn4.dmp",
    ];

    for filename in filenames {
        println!("\n---------------------------------------------");
        println!("Reading Candy Machine {:?}", filename);
        let data = load_account_dump(filename);
        let cm: CandyMachine = CandyMachine::try_deserialize(&mut data.as_slice()).unwrap();
        println!("Candy Machine: {}", filename);
        println!("Items Available: {:?}", cm.data.items_available);
        println!("Items Redeemed: {:?}", cm.items_redeemed);

        holaplex_indexer::geyser::programs::candy_machine::dump_cm_config_lines(
            &data,
            cm.data.items_available as usize,
        );

        holaplex_indexer::geyser::programs::candy_machine::check_for_bitmask_containing_n_ones(
            &data,
            cm.data.items_available as usize,
            cm.items_redeemed,
        );
    }

    assert_eq!(1, 2);
}
