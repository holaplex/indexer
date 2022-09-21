# `holaplex-indexer`
*A Solana indexer providing fast, accurate account information*

## Architecture

As a message producer, the Holaplex Indexing Service leverages the
[geyser-plugin-interface](https://github.com/solana-labs/solana/tree/master/geyser-plugin-interface)
to send accounts data directly to a RabbitMQ instance. As a message consumer,
the indexer consumes these account messages, deserializes them and inserts them
into a PostgreSQL database. Each account needs its own processor, schema and
model.

This dataset is derived entirely from the messages produced by a validator. This
supports a unidirectional dataflow. All data goes directly to the Solana
blockchain before it is saved in any off chain storage.

![](https://ipfs.cache.holaplex.com/bafkreiceois7frablbcdhiw4573m53rmhboadd5a2tkiw2mkle2el5udke)

### Components

- Solana Geyser plugin, responsible for sending data to our queue system
- RabbitMQ consumers, responsible for parsing messages and routing them to the
  proper processor
- PostgreSQL database, saves the deserialized data
- GraphQL Crate - serves the PostgreSQL data


### Data Indexed

Currently, the indexer covers the following Solana programs:

- [x] Holaplex wallet graph program
- [x] Metaplex program
- [x] Metaplex auction program
- [x] Metaplex auction house program
- [x] Metaplex candy machine program
- [x] Metaplex metadata program
- [x] SPL token program

Additionally, the following off-chain data is also indexed:

- [x] Holaplex storefronts
- [x] Holaplex marketplaces
- [x] Metaplex JSON metadata

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

## Database Connections

All indexer crates attempt to connect to the database by reading a Postgres URI
from one of three environment variables:

 - `DATABASE_READ_URL` is used by the GraphQL server to identify a read-only
   database.
 - `DATABASE_WRITE_URL` is used by all indexer services to identify a writable
   database.
 - `DATABASE_URL` is used as a fallback by all crates, and is assumed (but not
   guaranteed) to be writeable.

For debug builds the `.env*` files provided in the repository will provide a
default connection string pointed at the database defined in
`docker-compose.yml`.  For production builds the database must be manually
configured according to the environment variables above.

## Running the Indexer Cluster

The indexer consists of four services run by two binaries and a Geyser plugin.
All services are connected via a common RabbitMQ node.

### Geyser plugin setup

To build the Geyser plugin,clone this repo https://github.com/holaplex/indexer-geyser-plugin.git and use the following build command:

```sh
$ cargo build -pholaplex-indexer-rabbitmq-geyser
```

This will produce a build artifact named `libholaplex-indexer-rabbitmq-geyser`
with the appropriate file extension for a dynamic library for the host system
(i.e. `.dll`, `.dylib`, or `.so`).  This plugin can then be used with a Solana
validator.  A sample Geyser JSON configuration for the plugin can be found in
`crates/geyser-rabbitmq/sample_config.json`.

### Launching the services

Once the plugin is up and running, the three indexer consumer services can be
launched to process messages from the validator.  The consumers can be launched
as follows:

```sh
$ cargo run --bin holaplex-indexer-geyser --features geyser &
$ cargo run --bin holaplex-indexer-http --features http -- --entity metadata-json &
$ cargo run --bin holaplex-indexer-http --features http -- --entity store-config &
```

All services will need to be configured to run with the same settings that the
Geyser plugin was configured with, otherwise they will receive no messages or
simply fail to start.

## Running the GraphQL Server

### Configuration

The server binds to the address `[::]:3000` by default.  To change this, set the
`-p` argument/`PORT` environment variable or the `--addr` argument/`ADDRESS`
environment variable.  

To see more options for the server, run the following:

```sh
$ cargo run --bin holaplex-indexer-graphql -- --help
```

### Startup

To launch the GraphQL server, simply run the following:

```sh
$ cargo run --bin holaplex-indexer-graphql
```

### Contributing

Before pushing branch changes, run the following (or add it to your Git
pre-push hook) to check for problems, style errors, and schema issues:

```sh
$ scripts/pre-push.sh
```
