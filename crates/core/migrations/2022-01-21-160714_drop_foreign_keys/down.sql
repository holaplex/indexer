alter table metadata_creators add foreign key (metadata_address) references metadatas (address);
alter table listing_metadatas add  foreign key (listing_address) references listings (address);
alter table listing_metadatas add foreign key (metadata_address) references metadatas (address);
alter table editions add foreign key (metadata_address) references master_editions (address);
alter table editions add foreign key (parent_address) references master_editions (address);
alter table bids add foreign key (listing_address) references listings (address);
alter table listings add foreign key (store_owner) references storefronts (owner_address);