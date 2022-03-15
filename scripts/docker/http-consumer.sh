#!/bin/sh

./bin/metaplex-indexer-http --entity store-config --sender mainnet &
./bin/metaplex-indexer-http --entity metadata-json --sender mainnet
