create table reward_payouts (
  purchase_ticket varchar(48) primary key,
  metadata varchar(48) not null,
  reward_center varchar(48) not null,
  buyer varchar(48) not null,
  buyer_reward numeric not null,
  seller varchar(48) not null,
  seller_reward numeric not null,
  created_at timestamp not null,
  slot bigint not null,
  write_version bigint not null
);