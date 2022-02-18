# `Holaplex-Indexing-Service`


## Architecture
As a message producer, the Holaplex Indexing Service leverages the [accountsdb-plugin-interface](https://github.com/solana-labs/solana/tree/master/accountsdb-plugin-interface) to send accounts data directly to a RabbitMQ instance. As a message consumer, the indexer consumes these account messages, deserializes them and inserts them into a PostgreSQL database. Each account needs its own processor, schema and model.

This dataset is derived entirely from the messages produced by a validator. This supports a unidirectional dataflow. All data goes directly to the solana blockchain before it is saved in any off chain storage.

![](https://ipfs.cache.holaplex.com/bafkreiceois7frablbcdhiw4573m53rmhboadd5a2tkiw2mkle2el5udke)

### Components
- AccountsDB plugin, responsible for sending data to our queue system
- RabbitMQ Consumer, responsible for parsing messages and routing them to the proper processor
- PostgreSQL database, saves the deserialized data
- GrapqhQL Crate - serves the PostgreSQL data



Indexed Programs:

- [X] Metaplex Auctions
- [X] Metaplex Auction bids
- [X] Metaplex NFTs
- [X] Metaplex NFT json metadata
- [X] Metaplex Auction houses



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
$ cargo run --bin metaplex-indexer-graphql -- --help
```

### `graph`

To run the GraphQL Server, execute the following command:

```sh
$ cargo run --bin metaplex-indexer-graphql
```


