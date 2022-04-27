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

RUN cargo build --profile docker \
  --features " \
    holaplex-indexer/geyser, \
    holaplex-indexer/http \
    holaplex-indexer/search \
  " \
  --bin holaplex-indexer-geyser \
  --bin holaplex-indexer-http \
  --bin holaplex-indexer-legacy-storefronts \
  --bin holaplex-indexer-search \
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

FROM base AS geyser-consumer

COPY --from=build build/bin/holaplex-indexer-geyser bin/
COPY --from=build build/scripts/docker/geyser-consumer.sh startup.sh

FROM base AS http-consumer

COPY --from=build build/bin/holaplex-indexer-http bin/
COPY --from=build build/scripts/docker/http-consumer.sh startup.sh

FROM base AS legacy-storefronts

COPY --from=build build/bin/holaplex-indexer-legacy-storefronts bin/
COPY --from=build build/scripts/docker/legacy-storefronts.sh startup.sh

FROM base AS search-consumer

COPY --from=build build/bin/holaplex-indexer-search bin/
COPY --from=build build/scripts/docker/search-consumer.sh startup.sh

FROM base AS graphql

COPY --from=build build/bin/holaplex-indexer-graphql bin/
COPY --from=build build/scripts/docker/graphql.sh startup.sh
