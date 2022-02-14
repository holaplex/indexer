drop table auction_caches;
drop table auction_datas;
drop table auction_datas_ext;

create table listings (
  address                 varchar(48) primary key not null,
  ends_at                 timestamp,
  created_at              timestamp   not null,
  ended                   boolean     not null,
  authority               varchar(48) not null,
  token_mint              varchar(48) not null,
  store_owner             varchar(48) not null,
  highest_bid             bigint,
  end_auction_gap         timestamp,
  price_floor             bigint,
  total_uncancelled_bids  integer,
  gap_tick_size           integer,
  instant_sale_price      bigint,
  name                    text        not null,
  last_bid_time           timestamp
);
