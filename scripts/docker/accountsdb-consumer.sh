#!/bin/sh

./bin/metaplex-indexer-accountsdb --network=mainnet --startup=all &
./bin/metaplex-indexer-accountsdb --network=mainnet --startup=normal
