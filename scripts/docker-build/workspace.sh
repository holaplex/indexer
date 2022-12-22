#!/bin/bash

set -e

cargo build --locked \
  --profile docker \
  --features " \
    holaplex-indexer/http, \
    holaplex-indexer/job-runner, \
    holaplex-indexer/search, \
  " \
  --bin burn-fix \
  --bin dolphin-stats \
  --bin holaplex-indexer-dispatcher \
  --bin holaplex-indexer-http \
  --bin holaplex-indexer-job-runner \
  --bin holaplex-indexer-search \
  --bin holaplex-indexer-migrator \
  --bin holaplex-indexer-graphql \
  --bin moonrank-collections-indexer
