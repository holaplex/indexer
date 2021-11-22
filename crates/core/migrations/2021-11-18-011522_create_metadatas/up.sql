create table metadatas (
  address                   bytea   unique primary key not null,
  name                      text    not null,
  symbol                    text    not null,
  uri                       text    not null,
  seller_fee_basis_points   integer not null,
  update_authority_address  bytea   not null,
  mint_address              bytea   not null,
  primary_sale_happened     boolean default false,
  is_mutable                boolean default false,
  edition_nonce             integer null
);

create table metadata_creators (
  metadata_address  bytea   not null,
  creator_address   bytea   not null,
  share             integer not null,
  verified          boolean default false,

  primary key (metadata_address, creator_address),
  foreign key (metadata_address) references metadatas (address)
);

create index on metadata_creators (metadata_address);

create table listing_metadatas (
  listing_address   bytea not null,
  metadata_address  bytea not null,

  primary key (listing_address, metadata_address),
  foreign key (listing_address) references listings (address),
  foreign key (metadata_address) references metadatas (address)
);

create index on listing_metadatas (listing_address);
