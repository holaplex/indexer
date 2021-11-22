create table listings (
  address                 bytea     unique primary key not null,
  ends_at                 timestamp null,
  created_at              timestamp not null,
  ended                   boolean   not null,
  authority               bytea     not null,
  token_mint              bytea     not null,
  store                   bytea     not null,
  last_bid                bigint    null,
  end_auction_gap         timestamp null,
  price_floor             bigint    null,
  total_uncancelled_bids  integer   default 0,
  gap_tick_size           integer   null,
  instant_sale_price      bigint    null,
  name                    text      not null
);
