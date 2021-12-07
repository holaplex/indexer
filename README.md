# `metaplex-indexer`

An off-chain indexer for Metaplex stores

## Getting started

To set up a development environment, you will need rustup, Cargo, Docker, `docker-compose`, and the
Diesel CLI. Specifically, you will need `diesel_cli` installed with the `postgres` feature, which
can be done like so:

Installing diesel will require `libpq` to be on your system (`brew install
libpq` on mac). Also `brew install postgresql` if you don't already have it.

Then:

```sh
$ cargo install diesel_cli --no-default-features --features postgres
```

Once you have the requisite dependencies, you can get set up by running:

```sh
$  brew services start postgresql
$ ./start-developing.sh
```

## Running `indexer`

To run the indexer, simply enter the repository root and run:

```sh
$ cargo run --bin metaplex-indexer
```

## Running `rpc`

To run the RPC server, run the following (also from the repository root):

```sh
$ cargo run --bin metaplex-indexer-rpc
```

### If port `3000` is already in use

The `PORT` environment variable can be set to change the port `rpc` listens on:

```sh
$ PORT=3001 cargo run --bin metaplex-indexer-rpc
```

