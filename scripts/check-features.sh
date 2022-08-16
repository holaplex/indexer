#!/bin/bash

set -e
shopt -s globstar
cd "$(dirname "$0")/.."

function cc() {
  m="$1"
  shift 1
  cargo "${cmdbase[@]}" --manifest-path "$m" --no-default-features "$@"
}

function ccr() {
  cc "$@" --profile=release-lite
}

cmdbase=(check)
release=''
passes=(1)

while getopts cr2 opt; do
  case "$opt" in
    c) cmdbase=(clippy --no-deps) ;;
    r) release=1 ;;
    2) passes=({1..2}) ;;
  esac
done

for i in "${passes[@]}"; do
  echo $'\x1b[1m'":: Pass $i"$'\x1b[m'

  for f in **/Cargo.toml; do
    json="$(cargo read-manifest --manifest-path "$f" 2>/dev/null)" || continue

    name="$(jq -r .name <<<"$json")" || continue

    [[ -n "$name" ]] || continue
    echo $'\x1b[1m'" => $name"$'\x1b[m'

    echo "  -> No features"
    cc "$f"

    if [[ -n "$release" ]]; then
      echo "  -> No features (release)"
      ccr "$f"
    fi

    features="$(jq -r '.features | keys | join("\n")' <<<"$json")"

    for feat in $features; do
      [[ -n "$feat" ]] || continue

      args=("$f" --features "$feat")

      echo "  -> $feat"
      cc "${args[@]}"

      if [[ -n "$release" ]]; then
        echo "  -> $feat (release)"
        ccr "${args[@]}"
      fi
    done
  done
done
