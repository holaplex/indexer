#!/bin/bash

set -e

cd "$(dirname "$0")"

for f in '' .dev .local; do
  f="./.env$f"

  if [[ -f "$f" ]]; then
    echo "Loaded $f" >&2
    . "$f"
  fi
done

export DATABASE_NAME
export DATABASE_PASSWD
export DATABASE_URL

docker-compose up -d db

if ! command -v diesel >/dev/null; then
  echo $'Diesel CLI is missing - install it with:\n  cargo install diesel_cli --no-default-features --features postgres' >&2
  exit 1
fi

cd crates/core
diesel setup

