CREATE TABLE collections (
    id text NOT NULL PRIMARY KEY,
    image text NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    twitter_url text,
    discord_url text,
    website_url text,
    magic_eden_id text,
    verified_collection_address varchar(48),
    pieces bigint NOT NULL,
    verified bool NOT NULL,
    go_live_at timestamp not null,
    created_at timestamp NOT NULL,
    updated_at timestamp NOT NULL
);

create index on collections (verified_collection_address);
create index on collections (updated_at);
create index on collections (magic_eden_id);

CREATE TABLE collection_mints (
    collection_id text NOT NULL,
    mint varchar(48) NOT NULL,
    name text NOT NULL,
    image text NOT NULL,
    created_at timestamp NOT NULL,
    rank bigint NOT NULL,
    rarity numeric NOT NULL,
    PRIMARY KEY (collection_id, mint)
);

create index  on collection_mints (collection_id);
create index  on collection_mints (mint);

CREATE TABLE collection_mint_attributes (
    mint varchar(48) NOT NULL,
    attribute text NOT NULL,
    value text NOT NULL,
    value_perc numeric NOT NULL,
    PRIMARY KEY (mint, attribute, value)
);

create index  on collection_mint_attributes (mint);
