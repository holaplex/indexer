#!/bin/sh

./bin/metaplex-indexer-geyser --network=mainnet --startup=all &
./bin/metaplex-indexer-geyser --network=mainnet --startup=normal
