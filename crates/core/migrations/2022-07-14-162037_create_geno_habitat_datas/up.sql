create table geno_habitat_datas (
  address                         varchar(48) primary key,
  habitat_mint                    varchar(48) not null,
  level                           smallint    not null,
  element                         smallint    not null,
  genesis                         bool        not null,
  renewal_timestamp               timestamp   not null,
  expiry_timestamp                timestamp   not null,
  next_day_timestamp              timestamp   not null,
  crystals_refined                smallint    not null,
  harvester                       bytea       not null,
  ki_harvested                    bigint      not null,
  seeds_spawned                   bool        not null,
  is_sub_habitat                  bool        not null,
  parent_habitat                  varchar(48) null,
  sub_habitat_0                   varchar(48) null,
  sub_habitat_1                   varchar(48) null,
  harvester_royalty_bips          integer     not null,
  harvester_open_market           bool        not null,
  total_ki_harvested              bigint      not null,
  total_crystals_refined          bigint      not null,
  terraforming_habitat            varchar(48) null,
  active                          bool        not null,
  durability                      integer     not null,
  habitats_terraformed            integer     not null,
  sequence                        bigint      not null,
  guild                           integer     null,
  sub_habitat_cooldown_timestamp  timestamp   not null,
  slot                            bigint      not null,
  write_version                   bigint      not null
);

create table geno_rental_agreements (
  habitat_address     varchar(48) primary key,
  alchemist           varchar(48) null,
  rental_period       bigint      not null,
  rent                bigint      not null,
  rent_token          varchar(48) not null,
  rent_token_decimals smallint    not null,
  last_rent_payment   timestamp   not null,
  next_payment_due    timestamp   not null,
  grace_period        bigint      not null,
  open_market         bool        not null,
  slot                bigint      not null,
  write_version       bigint      not null,

  foreign key (habitat_address) references geno_habitat_datas (address)
);

create trigger geno_habitat_datas_check_slot_wv
before update on geno_habitat_datas for row
execute function check_slot_wv();

create trigger geno_rental_agreements_check_slot_wv
before update on geno_rental_agreements for row
execute function check_slot_wv();
