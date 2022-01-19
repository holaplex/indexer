create table token_accounts (
  address       varchar(48) primary key not null,
  mint_address  varchar(48) not null,
  owner_address varchar(48) not null,
  amount        bigint      default 0,
  updated_at    timestamp   not null
);
