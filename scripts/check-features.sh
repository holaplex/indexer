#!/bin/bash

set -e
shopt -s globstar
cd "$(dirname "$0")/.."

function puts() {
  if [[ -t 1 ]]; then
    echo "$@"
  else
    sed 's/\x1b\[[^m]*m//g' <<<"$@"
  fi
}

function putsc() {
  puts $'\x1b'"[$1m""$2"$'\x1b[m'
}

function cc() {
  m="$1"
  shift 1
  cmd=(cargo "${cmdbase[@]}" --manifest-path "$m" --no-default-features "$@")

  if [[ -n "$dry_run" ]]; then
    puts "${cmd[@]}"
  else
    "${cmd[@]}"
  fi
}

function ccr() {
  cc "$@" --profile=release-lite
}

cmdbase=(check)
dry_run=''
release=''
passes=(1)

while getopts cnr2 opt; do
  case "$opt" in
    c) cmdbase=(clippy --no-deps) ;;
    n) dry_run=1 ;;
    r) release=1 ;;
    2) passes=({1..2}) ;;
  esac
done

for i in "${passes[@]}"; do
  putsc 1 ":: Pass $i"

  for f in **/Cargo.toml; do
    json="$(cargo read-manifest --manifest-path "$f" 2>/dev/null)" || continue

    features="$(jq -r '.features | keys | join("\n")' <<<"$json")"
    targets="$(jq -r '.targets | keys | join("\n")' <<<"$json")"

    for tgt in $targets; do
      tj="$(jq -r ".targets[$tgt]" <<<"$json")"

      name="$(jq -r '.name' <<<"$tj")"
      kinds="$(jq -r '.kind | join("\n")' <<<"$tj")"
      req_features="$(jq -r '.["required-features"] | join(",")?' <<<"$tj")"

      if [[ -n "$req_features" ]]; then
        tgt_flags=(--features "$req_features")
      else
        tgt_flags=()
      fi

      for kind in $kinds; do
        putsc 1 " => $name ($kind)"

        case "$kind" in
          lib) kind_flags=(--lib) ;;
          *) kind_flags=("--$kind" "$name") ;;
        esac

        cc_flags=("$f" "${tgt_flags[@]}" "${kind_flags[@]}")

        putsc '38;5;4' '  -> No features'
        cc "${cc_flags[@]}"

        if [[ -n "$release" ]]; then
          putsc '38;5;5' '  -> No features (release)'
          ccr "${cc_flags[@]}"
        fi

        for feat in $features; do
          [[ -n "$feat" ]] || continue

          cc_feat_flags=("${cc_flags[@]}" --features "$feat")

          putsc '38;5;4' "  -> $feat"
          cc "${cc_feat_flags[@]}"

          if [[ -n "$release" ]]; then
            putsc '38;5;5' "  -> $feat (release)"
            ccr "${cc_feat_flags[@]}"
          fi
        done
      done
    done
  done
done
