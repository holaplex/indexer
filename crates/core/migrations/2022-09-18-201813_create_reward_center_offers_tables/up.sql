create table reward_center_offers (
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
  purchase_ticket_address bytea null,

  foreign key (reward_center_address) references reward_centers (address),
  foreign key (purchase_ticket_address) references reward_center_purchase_tickets (address)
)