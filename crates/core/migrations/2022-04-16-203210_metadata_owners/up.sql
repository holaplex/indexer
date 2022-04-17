create table if not exists current_metadata_owners (
    mint_address            varchar(48) not null primary key,
    owner_address           varchar(48) not null,
    token_account_address   varchar(48) not null,
    slot                    bigint      not null
);

create index if not exists metadata_owners_owner_addr_index on
  current_metadata_owners using hash (owner_address);