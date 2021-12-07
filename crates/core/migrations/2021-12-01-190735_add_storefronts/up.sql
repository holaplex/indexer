create table storefronts (
  owner_address varchar(48) unique primary key not null,
  subdomain     text        not null,
  title         text        not null,
  description   text        not null,
  favicon_url   text        not null,
  logo_url      text        not null
);

delete from listing_metadatas;
delete from listings;

alter table listings
add foreign key (store_owner) references storefronts (owner_address);
