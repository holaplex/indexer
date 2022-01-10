# `metaplex-indexer`

An off-chain indexer for Metaplex programs.

Available Indexes:

- [X] Metaplex Auctions and bids
- [X] Metaplex NFT for Auctions
- [ ] Metaplex Auction houses
- [ ] Metaplex NFTs by creator 

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

Once you have the required dependencies, you can get get developing by running the datastore for the indexer within a container in the background. After the Postgres DB is running start whichever serves you are looking to develop on:

```sh
$ ./start-developing.sh
```

## Running `indexer`

To run the indexer, simply enter the repository root and run:

```sh
$ cargo run --bin metaplex-indexer
```

## Running HTTP Servers

If port `3000` is already in use the `PORT` environment variable can be used chang the listener port of the servers:

### `rpc`

To run the RPC server, run the following (also from the repository root):

```sh
$ cargo run --bin metaplex-indexer-rpc
```

### `graph`

To run the GraphQL Server, execute the following command:

```sh
$ cargo run --bin metaplex-graph-server
```


