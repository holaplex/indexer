create table token_transfers (
  owner_from      varchar(48),     
  owner_to        varchar(48),  
  mint_address    varchar(48),     
  transferred_at  timestamp not null,

  primary key (owner_from, owner_to, mint_address, transferred_at)
);
