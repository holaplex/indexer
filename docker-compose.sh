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

docker-compose "$@"
