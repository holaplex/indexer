#!/bin/sh

bin/holaplex-indexer-http --entity store-config --sender mainnet &
bin/holaplex-indexer-http --entity metadata-json --sender mainnet
