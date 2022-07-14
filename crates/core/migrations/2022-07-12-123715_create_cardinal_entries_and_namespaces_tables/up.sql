create table cardinal_entries (
  address                       varchar(48)     primary key,
  namespace                     varchar(48)     not null,
  name                          text            not null,
  data                          varchar(48),
  reverse_entry                 varchar(48),
  mint                          varchar(48)     not null,
  is_claimed                    bool            not null,
  slot                          bigint          not null,
  write_version                 bigint          not null
);

create table cardinal_namespaces (
  address                       varchar(48)     primary key,
  name                          text            not null,
  update_authority              varchar(48)     not null,
  rent_authority                varchar(48)     not null,
  approve_authority             varchar(48),
  schema                        smallint        not null,
  payment_amount_daily          bigint          not null,
  payment_mint                  varchar(48)     not null,
  min_rental_seconds            bigint          not null,
  max_rental_seconds            bigint,
  transferable_entries          bool            not null,
  slot                          bigint          not null,
  write_version                 bigint          not null
);

create trigger cardinal_entries_check_slot_wv
before update on cardinal_namespaces for row
execute function check_slot_wv();

create trigger cardinal_namespaces_check_slot_wv
before update on cardinal_namespaces for row
execute function check_slot_wv();
