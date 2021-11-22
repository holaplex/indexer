create table master_editions (
  address     bytea   unique primary key not null,
  supply      bigint  not null,
  max_supply  bigint
);

create table editions (
  address         bytea   unique primary key not null,
  parent_address  bytea   not null,
  edition         bigint  not null,

  foreign key (parent_address) references master_editions (address)
);
