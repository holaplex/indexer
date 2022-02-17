create table token_transfers (
  owner_from      varchar(48),     
  to_owner        varchar(48),  
  mint_address    varchar(48),     
  transfered_at   timestamp not null,

  primary key (owner_from,to_owner,mint_address,transfered_at)
);

