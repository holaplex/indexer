FROM rust:1.58.1-slim-bullseye AS build
WORKDIR /metaplex-indexer

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
    metaplex-indexer/accountsdb, \
    metaplex-indexer/http \
  " \
  --bin metaplex-indexer-accountsdb \
  --bin metaplex-indexer-http \
  --bin metaplex-indexer-legacy-storefronts \
  --bin metaplex-indexer-graphql

COPY scripts scripts

RUN scripts/strip-bins.sh target/docker bin

FROM debian:bullseye-slim AS base
WORKDIR /metaplex-indexer

RUN apt-get update -y && \
  apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl1.1 \
  && \
  rm -rf /var/lib/apt/lists/*

RUN mkdir -p bin

CMD ["./startup.sh"]

FROM base AS accountsdb-consumer

COPY --from=build metaplex-indexer/bin/metaplex-indexer-accountsdb bin/
COPY --from=build metaplex-indexer/scripts/docker/accountsdb-consumer.sh startup.sh

FROM base AS http-consumer

COPY --from=build metaplex-indexer/bin/metaplex-indexer-http bin/
COPY --from=build metaplex-indexer/scripts/docker/http-consumer.sh startup.sh

FROM base AS legacy-storefronts

COPY --from=build metaplex-indexer/bin/metaplex-indexer-legacy-storefronts bin/
COPY --from=build metaplex-indexer/scripts/docker/legacy-storefronts.sh startup.sh

FROM base AS graphql

COPY --from=build metaplex-indexer/bin/metaplex-indexer-graphql bin/
COPY --from=build metaplex-indexer/scripts/docker/graphql.sh startup.sh
