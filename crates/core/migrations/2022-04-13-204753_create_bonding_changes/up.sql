CREATE TABLE IF NOT EXISTS bonding_changes (
  address varchar(48) NOT NULL,
  insert_ts timestamp without time zone NOT NULL,
  slot bigint not null,
  current_reserves_from_bonding bigint not null,
  current_supply_from_bonding bigint not null,
  primary key(address, slot)
);

CREATE INDEX ON bonding_changes(address, insert_ts);
CREATE INDEX ON bonding_changes(address, slot);
