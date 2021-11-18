#!/bin/bash

set -e

for f in '' .dev .local; do
  f="./.env$f"

  if [[ -f "$f" ]]; then
    echo "Loaded $f" >&2
    . "$f"
  fi
done

export DATABASE_URL

cd "$(dirname "$0")"/crates/core

diesel $@
