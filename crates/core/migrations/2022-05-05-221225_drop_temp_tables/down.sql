create table if not exists temp_attributes (
  metadata_address        varchar(48) not null,
  value                   text,
  trait_type              text,
  id                      uuid        primary key default gen_random_uuid(),
  first_verified_creator  varchar(48)
);

create table if not exists token_accounts (
  address       varchar(48) primary key,
  mint_address  varchar(48) not null,
  owner_address varchar(48) not null,
  amount        bigint      not null default 0,
  updated_at    timestamp   not null default now(),
  slot          bigint
);

create trigger set_token_account_updated_at
before update on token_accounts
for each row
execute procedure trigger_set_updated_at_timestamp();

-- Deliberately not recreating indices

