#!/bin/bash

set -e

cd "$(dirname "$0")/.."

build_flags=(--workspace --lib --bins --tests --all-features)

[[ -z "$CARGO" ]] && CARGO=cargo

diff --unified <(./diesel.sh print-schema) crates/core/src/db/schema.rs
"$CARGO" fmt --all --check
"$CARGO" clippy "${build_flags[@]}"
"$CARGO" doc "${build_flags[@]}"
"$CARGO" build "${build_flags[@]}"
"$CARGO" test "${build_flags[@]}"
