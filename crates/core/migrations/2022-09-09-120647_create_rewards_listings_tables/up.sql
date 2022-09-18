create table rewards_listings (
  address varchar(48) primary key,
  is_initialized boolean not null,
  reward_center_address varchar(48) not null,
  seller varchar(48) not null,
  metadata varchar(48) not null,
  price bigint not null,
  token_size bigint not null,
  bump smallint not null,
  created_at timestamp not null,
  canceled_at timestamp null,
  purchase_ticket varchar(48) null,
  slot bigint not null default -1,
  write_version bigint not null
);

create trigger rewards_listings_check_slot_wv
before update on rewards_listings for row
execute function check_slot_wv();