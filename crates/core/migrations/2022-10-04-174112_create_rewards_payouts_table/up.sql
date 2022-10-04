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

create index reward_payouts_reward_center_idx on reward_payouts(reward_center);
create index reward_payouts_metadata_idx on reward_payouts(metadata);
create index reward_payouts_buyer_idx on reward_payouts(buyer);
create index reward_payouts_seller_idx on reward_payouts(seller);
create index reward_payouts_created_at_idx on reward_payouts(created_at);