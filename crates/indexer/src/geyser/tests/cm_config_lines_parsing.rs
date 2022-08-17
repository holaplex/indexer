use core::convert::Into;
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
};

use anchor_lang_v0_21::{AccountDeserialize, AnchorDeserialize};
use holaplex_indexer::geyser::programs::candy_machine::parse_cm_config_lines;
use mpl_candy_machine::{CandyMachine, CollectionPDA, ConfigLine, CONFIG_LINE_SIZE};
use solana_program::example_mocks::solana_sdk;

#[cgf(test)]
mod test {
    fn test_can_deser_candy_machine() {
        let filenames = [
            "candy_machines/FhrVJL4xKNmAY53Bm5XJNqJwvBomDuDH7HGDdicgbkZY.dmp",
            "candy_machines/AoHidoffmkL4xURViNgbA4YyeDw82FAYUZfomL3X5BoU.dmp",
            "candy_machines/piA76RvvmCt7UWEmJSBVA6xMoXqwvEAELwJoqeHK6i3.dmp",
            "candy_machines/CiBuYi3W3aVQbMWcjvfKBpwjHS6fViuuxQdSUUqkjkn4.dmp",
            "candy_machines/ACDPaQ3uGy33KsBKiUH4azDX4q7Nxk3QwW3trALEdFmB.dmp", /* this one has hidden settings */
        ];

        for filename in filenames {
            println!("\n---------------------------------------------");
            println!("Reading Candy Machine {:?}", filename);
            let data = load_account_dump(filename);
            let cm = CandyMachine::try_deserialize(&mut data.as_slice()).unwrap();
            assert_eq(1, 2);
            println!("Candy Machine: {}", filename);
            println!("Items Available: {:?}", cm.data.items_available);
            println!("Items Redeemed: {:?}", cm.items_redeemed);

            let results = parse_cm_config_lines(&data, cm.data.items_available as usize);

            let available_count = results.len();
            let mut taken_count = 0;
            for (config_line, idx, taken) in results.iter() {
                // println!(
                //     "idx: {} - taken: {} - name: {:?} uri: {:?}",
                //     *idx,
                //     *taken,
                //     config_line.name.trim_matches(char::from(0)),
                //     config_line.uri.trim_matches(char::from(0))
                // );
                if *taken {
                    taken_count += 1;
                }
            }

            // NOTE(will): why doesn't this work
            // let taken_count = results
            //     .iter()
            //     .map(|v| (v.2).copy())
            //     .filter(|t| t)
            //     .collect::<bool>()
            //     .len();

            println!("available_count: {}", available_count);
            println!("taken_count: {}", taken_count);

            if cm.data.hidden_settings.is_some() {
                assert_eq!(available_count, 0);
                assert_eq!(taken_count, 0);
            } else {
                assert_eq!(available_count, cm.data.items_available as usize);
                assert_eq!(taken_count, cm.items_redeemed);
            }
        }
    }
}
