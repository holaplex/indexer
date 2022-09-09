create table rewards_offers (
  address bytea unique primary key not null,
  is_initialized boolean not null,
  reward_center_address bytea not null,
  buyer bytea not null,
  metadata bytea not null,
  price bigint not null,
  token_size bigint not null,
  bump smallint not null,
  created_at bigint not null,
  canceled_at bigint null,
  purchased_at bigint null,
  reward_redeemed_at bigint null
)