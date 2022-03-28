#!/bin/sh

bin/holaplex-indexer-geyser --network=mainnet --startup=all &
bin/holaplex-indexer-geyser --network=mainnet --startup=normal
