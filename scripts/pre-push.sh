#!/bin/bash

set -e

cd "$(dirname "$0")/.."

lib_flags=(--workspace --lib --all-features)
bin_flags=(--workspace --bins --all-features)

[[ -z "$CARGO" ]] && CARGO=cargo

diff --unified <(./diesel.sh print-schema) crates/core/src/db/schema.rs

for m in Cargo.toml crates/geyser-consumer/Cargo.toml; do
  [[ -t 2 ]] && echo $'\x1b[1m'"Checking manifest $m..."$'\x1b[m'

  mp=(--manifest-path "$m")

  "$CARGO" fmt "${mp[@]}" --all --check

  # hack
  no_lib=''
  if [[ "$m" != Cargo.toml ]]; then
    no_lib=t
  fi

  if [[ -z "$no_lib" ]]; then
    "$CARGO" clippy "${mp[@]}" "${lib_flags[@]}" --no-deps
    "$CARGO" doc "${mp[@]}" "${lib_flags[@]}" --no-deps
    "$CARGO" build "${mp[@]}" "${lib_flags[@]}"
    "$CARGO" test "${mp[@]}" "${lib_flags[@]}"
  fi

  "$CARGO" clippy "${mp[@]}" "${bin_flags[@]}" --no-deps
  [[ -z "$no_lib" ]] || "$CARGO" doc "${mp[@]}" "${bin_flags[@]}" --no-deps
  "$CARGO" build "${mp[@]}" "${bin_flags[@]}"
  "$CARGO" test "${mp[@]}" "${bin_flags[@]}"
done
