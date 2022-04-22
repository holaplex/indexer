create table bonding_changes (
  address                       varchar(48) not null,
  insert_ts                     timestamp   not null,
  slot                          bigint      not null,
  current_reserves_from_bonding bigint      not null,
  current_supply_from_bonding   bigint      not null,
  primary key (address, slot)
);

create index on bonding_changes (address, insert_ts);
create index on bonding_changes (address, slot);
