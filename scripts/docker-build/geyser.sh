#!/bin/bash

set -e

cargo build --manifest-path crates/geyser-consumer/Cargo.toml \
  --locked \
  --profile docker \
  --bin holaplex-indexer-geyser
