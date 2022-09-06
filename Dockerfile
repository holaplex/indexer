FROM rust:1.58.1-slim-bullseye AS build
WORKDIR /build

RUN apt-get update -y && \
  apt-get install -y \
    libpq-dev \
    libssl-dev \
    libudev-dev \
    pkg-config \
  && \
  rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml ./

# Force rustup to install toolchain
RUN rustc --version

COPY crates crates
COPY Cargo.toml Cargo.lock ./

RUN cargo fetch --locked

RUN cargo build --locked \
  --profile docker \
  --features " \
    holaplex-indexer/geyser, \
    holaplex-indexer/http, \
    holaplex-indexer/job-runner, \
    holaplex-indexer/search, \
  " \
  --bin holaplex-indexer-dispatcher \
  --bin holaplex-indexer-geyser \
  --bin holaplex-indexer-http \
  --bin holaplex-indexer-job-runner \
  --bin holaplex-indexer-search \
  --bin holaplex-indexer-migrator \
  --bin holaplex-indexer-graphql

COPY scripts scripts

RUN scripts/strip-bins.sh target/docker bin

FROM debian:bullseye-slim AS base
WORKDIR /holaplex-indexer

RUN apt-get update -y && \
  apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl1.1 \
  && \
  rm -rf /var/lib/apt/lists/*

RUN mkdir -p bin

COPY .env .env.prod ./

CMD ["./startup.sh"]

FROM base AS dispatcher-base

COPY --from=build build/bin/holaplex-indexer-dispatcher bin/

FROM dispatcher-base AS dispatch-refresh-table

COPY --from=build build/scripts/docker/dispatch-refresh-table.sh startup.sh

FROM base AS geyser-consumer

COPY --from=build build/bin/holaplex-indexer-geyser bin/
COPY --from=build build/scripts/docker/geyser-consumer.sh startup.sh

FROM base AS http-consumer

COPY --from=build build/bin/holaplex-indexer-http bin/
COPY --from=build build/scripts/docker/http-consumer.sh startup.sh

FROM base AS job-runner

COPY --from=build build/bin/holaplex-indexer-job-runner bin/
COPY --from=build build/scripts/docker/job-runner.sh startup.sh

FROM base AS search-consumer

COPY --from=build build/bin/holaplex-indexer-search bin/
COPY --from=build build/scripts/docker/search-consumer.sh startup.sh

FROM base AS migrator

COPY --from=build build/bin/holaplex-indexer-migrator bin/
COPY --from=build build/scripts/docker/migrator.sh startup.sh

FROM base AS graphql

COPY --from=build build/bin/holaplex-indexer-graphql bin/
COPY --from=build build/scripts/docker/graphql.sh startup.sh
