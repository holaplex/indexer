# `metaplex-indexer`

An off-chain indexer for Metaplex programs.

Available Indexes:

- [X] Metaplex Auctions and bids
- [X] Metaplex NFT for Auctions
- [ ] Metaplex Auction houses
- [ ] Metaplex NFTs by creator

## Getting started

### Diesel

To set up a development environment, you will need `rustup`, Cargo, Docker,
`docker-compose`, and the Diesel CLI. Specifically, you will need `diesel_cli`
installed with the `postgres` feature, which can be done like so:

```sh
$ cargo install diesel_cli --no-default-features --features postgres
```

Installing diesel will require `libpq` to be on your system (`brew install
postgresql` on Mac).

### Migrating

Once you have the required dependencies, you can get started by running the
following script to initialize and migrate a containerized Postgres database in
the background:

```sh
$ ./start-developing.sh
```

## Running `indexer`

To run the indexer, simply enter the repository root and run:

```sh
$ cargo run --bin metaplex-indexer
```

This will perform a single scan of the requested data and quit.  There are
several configuration options available to change what is indexed, to see them
run the following:

```sh
$ cargo run --bin metaplex-indexer -- --help
```

## Running HTTP Servers

### Configuration

Both servers have some common configuration options.  For instance, both servers
run on port `3000` by default, but this can be changed with the `-p`
command-line flag or by setting the `PORT` environment variable.  To see more
options for each server, run one of the following:

```sh
$ cargo run --bin metaplex-indexer-rpc -- --help
$ cargo run --bin metaplex-indexer-graphql -- --help
```

### `rpc`

To run the RPC server, run the following (also from the repository root):

```sh
$ cargo run --bin metaplex-indexer-rpc
```

### `graph`

To run the GraphQL Server, execute the following command:

```sh
$ cargo run --bin metaplex-indexer-graphql
```


