FROM rust:1.64.0-slim-bullseye AS build-base
WORKDIR /build

RUN rustup toolchain remove 1.64.0

RUN apt-get update -y && \
  apt-get install -y \
    jq \
    libpq-dev \
    libssl-dev \
    libudev-dev \
    pkg-config \
  && \
  rm -rf /var/lib/apt/lists/*

COPY rust-toolchain.toml ./

# Force rustup to install toolchain
RUN rustc --version

FROM build-base AS skel

COPY scripts/install-skeleton.sh ./scripts/
RUN mkdir ../repo
COPY . ../repo
RUN scripts/install-skeleton.sh ../repo .
RUN scripts/install-skeleton.sh ../repo/crates/geyser-consumer crates/geyser-consumer
RUN scripts/install-skeleton.sh ../repo/crates/genostub crates/genostub

FROM build-base AS build

COPY --from=skel /build/Cargo.lock /build/Cargo.toml ./
COPY --from=skel /build/crates crates

RUN cargo fetch --locked
RUN cargo fetch --locked --manifest-path crates/geyser-consumer/Cargo.toml

COPY scripts/docker-build ./scripts/docker-build

RUN scripts/docker-build/workspace.sh
RUN scripts/docker-build/geyser.sh

COPY crates crates
COPY scripts/install-skeleton.sh ./scripts/
RUN scripts/install-skeleton.sh -t .
RUN scripts/install-skeleton.sh -t crates/geyser-consumer
RUN scripts/install-skeleton.sh -t crates/genostub

RUN scripts/docker-build/workspace.sh
RUN scripts/docker-build/geyser.sh

COPY scripts scripts

RUN scripts/strip-bins.sh target/docker bin
RUN scripts/strip-bins.sh crates/geyser-consumer/target/docker bin

FROM debian:bullseye-slim AS base
WORKDIR /opt/indexer

RUN apt-get update -y && \
  apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 \
    libssl1.1 \
  && \
  rm -rf /var/lib/apt/lists/*

RUN mkdir -p bin

COPY .env .env.prod ./

CMD ["./startup.sh"]

FROM base AS tools

COPY --from=build \
  build/bin/burn-fix \
  build/bin/dolphin-stats \
  build/bin/moonrank-collections-indexer \
  bin/
CMD ["false"]

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
