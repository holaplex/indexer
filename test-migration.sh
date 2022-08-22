#!/bin/bash

set -e

cd "$(dirname "$0")"

exec ./diesel.sh migration --migration-dir test_migrations "$@"
