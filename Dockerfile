FROM rust:1.58.1-slim
WORKDIR /metaplex-indexer

RUN apt-get update -y && \
  apt-get install -y \
    libpq-dev \
    libssl-dev \
    libudev-dev \
    pkg-config \
  && \
  rm -rf /var/lib/apt/lists/*

RUN rustup toolchain uninstall 1.58.1

COPY rust-toolchain.toml ./

# Force rustup to install toolchain
RUN rustc --version

ARG PORT
ENV PORT=$PORT

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

COPY crates crates
COPY Cargo.toml Cargo.lock ./

RUN cargo build --profile heroku \
  -pmetaplex-indexer \
  -pmetaplex-indexer-rpc \
  -pmetaplex-indexer-graphql

RUN strip target/heroku/metaplex-indexer && \
  strip target/heroku/metaplex-indexer-rpc && \
  strip target/heroku/metaplex-indexer-graphql && \
  mkdir bin && \
  mv target/heroku/metaplex-indexer \
    target/heroku/metaplex-indexer-rpc \
    target/heroku/metaplex-indexer-graphql \
    bin

RUN rm -rf target /usr/local/cargo/registry && \
  rustup toolchain uninstall \
    nightly-2021-11-09 \
    nightly \
    stable

RUN apt-get install -y libpq5 && \
  apt-get remove -y \
    libssl-dev \
    libpq-dev \
    libudev-dev \
    pkg-config \
  && \
  rm -rf /var/lib/apt/lists/*
