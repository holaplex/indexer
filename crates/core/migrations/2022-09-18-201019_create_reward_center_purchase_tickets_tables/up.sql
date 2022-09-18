create table reward_center_purchase_tickets (
  address bytea unique primary key not null,
  reward_center_address bytea not null,
  buyer bytea not null,
  seller bytea not null,
  metadata bytea not null,
  price bigint not null,
  token_size bigint not null,
  created_at bigint not null,

  foreign key (reward_center_address) references reward_centers (address)
)