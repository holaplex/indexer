create table metadata_collection_keys (
    metadata_address varchar(48),
    collection_address varchar(48),
    verified bool not null,
    primary key (metadata_address, collection_address)
);

create type token_standard 
as enum ('NonFungible', 'FungibleAsset', 'Fungible', 'NonFungibleEdition');

alter table metadatas
add column token_standard token_standard null;