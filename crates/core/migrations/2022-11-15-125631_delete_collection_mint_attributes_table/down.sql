CREATE TABLE collection_mint_attributes (
    mint varchar(48) NOT NULL,
    attribute text NOT NULL,
    value text NOT NULL,
    value_perc numeric NOT NULL,
    PRIMARY KEY (mint, attribute, value)
);

create index  on collection_mint_attributes (mint);