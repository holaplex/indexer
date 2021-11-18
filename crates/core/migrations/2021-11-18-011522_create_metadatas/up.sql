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
  address           bytea   unique primary key not null,
  metadata_address  bytea   not null,
  creator_address   bytea   not null,
  share             integer not null,
  verified          boolean default false,

  foreign key (metadata_address) references metadatas (address)
);

create table listing_metadatas (
  listing_address   bytea primary key not null,
  metadata_address  bytea not null,

  foreign key (listing_address) references listings (address),
  foreign key (metadata_address) references metadatas (address)
);
