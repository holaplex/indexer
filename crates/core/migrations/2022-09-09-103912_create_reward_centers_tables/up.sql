create table reward_centers (
  address             bytea     unique primary key not null,
  token_mint          bytea     not null,
  auction_house       bytea     not null,
  bump                smallint  not null
);

create table listing_reward_rules (
  reward_center_address               bytea    not null,
  seller_reward_payout_basis_points   smallint not null,
  payout_divider                      smallint not null,

  primary key (reward_center_address),
  foreign key (reward_center_address) references reward_centers (address)
);

create index on listing_reward_rules (reward_center_address);