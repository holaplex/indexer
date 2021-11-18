create table master_editions (
  address     bytea   unique primary key not null,
  supply      integer not null,
  max_supply  integer not null
);

create table editions (
  address         bytea   unique primary key not null,
  parent_address  bytea   not null,
  edition         integer not null,

  foreign key (parent_address) references master_editions (address)
);
