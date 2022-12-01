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

DOCKER_COMPOSE="docker-compose"

which podman-compose 2>&1 && ! which docker-compose 2>&1 && DOCKER_COMPOSE="podman-compose"

"$DOCKER_COMPOSE" "$@"
