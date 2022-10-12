CREATE TABLE collections (
    id text NOT NULL PRIMARY KEY,
    image text NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    verified_collection_address varchar(48),
    pieces bigint NOT NULL,
    verified bool NOT NULL,
    created_at timestamp NOT NULL,
    updated_at timestamp NOT NULL
);

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

CREATE TABLE collection_mint_attributes (
    mint varchar(48) NOT NULL,
    attribute text NOT NULL,
    value text NOT NULL,
    value_perc numeric NOT NULL,
    PRIMARY KEY (mint, attribute, value)
);

