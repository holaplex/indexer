#!/bin/bash

set -e

cd "$(dirname "$0")/.."

lib_flags=(--workspace --lib --all-features)
bin_flags=(--workspace --bins --all-features)

[[ -z "$CARGO" ]] && CARGO=cargo

diff --unified <(./diesel.sh print-schema) crates/core/src/db/schema.rs
"$CARGO" fmt --all --check
"$CARGO" clippy "${lib_flags[@]}" --no-deps
"$CARGO" doc "${lib_flags[@]}" --no-deps
"$CARGO" build "${lib_flags[@]}"
"$CARGO" test "${lib_flags[@]}"

"$CARGO" clippy "${bin_flags[@]}" --no-deps
"$CARGO" build "${bin_flags[@]}"
"$CARGO" test "${bin_flags[@]}"
