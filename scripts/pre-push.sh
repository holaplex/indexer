#!/bin/bash

set -e

cd "$(dirname "$0")/.."

lib_flags=(--workspace --lib)
build_flags=("${lib_flags[@]}")
build_flags+=(--bins)

test_flags=("$build_flags")
test_flags+=(--features test-internal)

[[ -z "$CARGO" ]] && CARGO=cargo

diff --unified <(./diesel.sh print-schema) crates/core/src/db/schema.rs
"$CARGO" fmt --all --check
"$CARGO" clippy "${build_flags[@]}" --no-deps
"$CARGO" doc "${lib_flags[@]}" --no-deps
"$CARGO" build "${build_flags[@]}"
"$CARGO" test "${test_flags[@]}"
