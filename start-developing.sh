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

function isready() {
  PGPASSWORD="$DATABASE_PASSWD" pg_isready \
    -d "$DATABASE_NAME" -h localhost -p 5337 -U postgres -t 10 \
    || return 1

  return 0
}

if ! isready; then
  echo "Waiting for Postgres to come online..."

  for i in {0..10}; do
    sleep 3
    isready || continue
    break
  done

  isready
fi

if ! command -v diesel >/dev/null; then
  echo $'Diesel CLI is missing - install it with:\n  cargo install diesel_cli --no-default-features --features postgres' >&2
  exit 1
fi

(cd crates/core && diesel setup)
