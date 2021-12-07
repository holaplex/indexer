# `metaplex-indexer`
An off-chain indexer for Metaplex stores

## Getting started

To set up a development environment, you will need rustup, Cargo, Docker, `docker-compose`, and the
Diesel CLI.  Specifically, you will need `diesel_cli` installed with the `postgres` feature, which
can be done like so:

```sh
$ cargo install diesel_cli --no-default-features --features postgres
```

Installing diesel will require `libpq` to be on your system.

Once you have the requisite dependencies, you can get set up by running:

```sh
$ ./start-developing.sh
```

## Running `indexer`

To run the indexer, simply enter the repository root and run:

```sh
$ cargo run --bin metaplex-indexer
```

## Running `rpc`

TODO
