create table rewards_purchase_tickets (
  address varchar(48) primary key,
  reward_center_address varchar(48) not null,
  buyer varchar(48) not null,
  seller varchar(48) not null,
  metadata varchar(48) not null,
  price bigint not null,
  token_size bigint not null,
  created_at timestamp not null,
  slot bigint not null default -1,
  write_version bigint not null
);

create trigger rewards_purchase_tickets_check_slot_wv
before update on rewards_purchase_tickets for row
execute function check_slot_wv();