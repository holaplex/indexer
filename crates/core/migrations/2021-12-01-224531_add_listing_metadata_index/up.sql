delete from listing_metadatas;

alter table listing_metadatas
add column metadata_index integer not null;
