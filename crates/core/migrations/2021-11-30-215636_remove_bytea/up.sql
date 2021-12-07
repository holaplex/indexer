-- First, clear data (it could be converted, but it's easier to just regenerate)
delete from editions;
delete from listing_metadatas;
delete from listings;
delete from master_editions;
delete from metadata_creators;
delete from metadatas;

-- Then, remove constraints and update column types
alter table editions
drop constraint editions_parent_address_fkey,
alter column address        type varchar(48),
alter column parent_address type varchar(48);

alter table listing_metadatas
drop constraint listing_metadatas_listing_address_fkey,
drop constraint listing_metadatas_metadata_address_fkey,
alter column listing_address  type varchar(48),
alter column metadata_address type varchar(48);

alter table listings
alter column address      type varchar(48),
alter column authority    type varchar(48),
alter column token_mint   type varchar(48),
alter column store_owner  type varchar(48);

alter table master_editions
alter column address type varchar(48);

alter table metadata_creators
drop constraint metadata_creators_metadata_address_fkey,
alter column metadata_address type varchar(48),
alter column creator_address  type varchar(48);

alter table metadatas
alter column address                  type varchar(48),
alter column update_authority_address type varchar(48),
alter column mint_address             type varchar(48);

-- Finally, add the constraints back
alter table editions
add foreign key (parent_address) references master_editions (address);

alter table listing_metadatas
add foreign key (listing_address) references listings (address),
add foreign key (metadata_address) references metadatas (address);

alter table metadata_creators
add foreign key (metadata_address) references metadatas (address);
