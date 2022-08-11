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
    let data_dir = get_data_dir();
    let filename = "AoHidoffmkL4xURViNgbA4YyeDw82FAYUZfomL3X5BoU.dmp";
    // let filename = "FhrVJL4xKNmAY53Bm5XJNqJwvBomDuDH7HGDdicgbkZY.dmp";
    let data = load_account_dump(filename);
    let cm: CandyMachine = CandyMachine::try_deserialize(&mut data.as_slice()).unwrap();
    println!("Candy Machine: {}", filename);
    println!("Items Available: {:?}", cm.data.items_available);
    println!("Items Redeemed: {:?}", cm.items_redeemed);
    // println!("Symbol: {:?}", cm.data.symbol);
    let config_lines = holaplex_indexer::geyser::programs::candy_machine::parse_cm_config_lines(
        &data,
        cm.data.items_available as usize,
    )
    .unwrap();

    for (idx, line) in config_lines.iter().enumerate() {
        match line {
            Some(config_line) => {
                println!(
                    "{:?} : {:?},{:?}",
                    idx,
                    config_line.name.trim_matches(char::from(0)),
                    config_line.uri.trim_matches(char::from(0))
                )
            },
            None => println!("{:?} : ", idx),
        }
    }
    // assert_matches!(Ok(_), config_lines);

    // assert_eq(cm.data.items_available, 75);
    // assert_eq(cm.data.symbol, "CLHP\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}");
    // assert_eq(cm.data.)

    // holaplex_indexer::geyser::programs::candy_machine::parse_cm_config_lines(&data);
    // let (cm, config_lines) = deser_cm;
    // let candy_machine = CandyMachine::try_deserialize(&mut data.as_slice())?;
    println!("{:?}", data_dir);
    println!("{:?}", data.len());
    assert_eq!(1, 2);
}
