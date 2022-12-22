#!/bin/bash

set -e
cd "$(dirname "$0")/.."

touch=''

while getopts t opt; do
  case "$opt" in
    t) touch=t ;;
  esac
done

shift $(( OPTIND - 1 ))

from="$1"
shift 1
to="$1"
shift 1 || to="$from"

realfrom="$(realpath "$from")"
realto="$(realpath "$to")"

function ins() {
  local src="$(realpath "$1")"
  shift 1

  [[ "$src" == "$realfrom"* ]]


  local sdir="$(dirname "$src")/"
  local ddir="$realto/${sdir#$realfrom/}"

  mkdir -p -- "$ddir"

  if [[ -n "$touch" ]]; then
    touch "$ddir/$(basename "$src")"
    return
  fi

  local proxy="$1"
  if shift 1; then
    install -T -- "$proxy" "$ddir/$(basename "$src")"
  else
    install -t"$ddir" -- "$src"
  fi
}

for f in Cargo.toml Cargo.lock; do
  ins "$from/$f"
done

json="$(cargo metadata --format-version=1 --no-deps --manifest-path "$from/Cargo.toml")"

if [[ -z "$touch" ]]; then
  function drop_temp() { rm "$main"; }
  trap drop_temp EXIT

  main="$(mktemp /tmp/main.rs.XXXX)"
  echo 'fn main() {}' >"$main"
fi

for pkg in $(jq -r '.packages | keys | join("\n")' <<<"$json"); do
  pj="$(jq ".packages[$pkg]" <<<"$json")"

  ins "$(jq -r '.manifest_path' <<<"$pj")"

  for tgt in $(jq -r '.targets | keys | join("\n")' <<<"$pj"); do
    ins "$(jq -r ".targets[$tgt].src_path" <<<"$pj")" "$main"
  done
done
