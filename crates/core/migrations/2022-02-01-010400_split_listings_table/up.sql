create table auction_caches (
  address         varchar(48) primary key not null,
  store_address   varchar(48) not null,
  timestamp       timestamp not null,
  auction_data    varchar(48) not null,
  auction_ext     varchar(48) not null,
  vault           varchar(48) not null,
  auction_manager varchar(48) not null
);

create table auction_datas (
  address                 varchar(48) primary key not null,
  ends_at                 timestamp,
  authority               varchar(48),
  token_mint              varchar(48),
  store_owner             varchar(48),
  highest_bid             bigint,
  end_auction_gap         timestamp,
  price_floor             bigint,
  total_uncancelled_bids  integer,
  last_bid_time           timestamp
);

create table auction_datas_ext (
  address             varchar(48) primary key not null,
  gap_tick_size       integer,
  instant_sale_price  bigint,
  name                text not null
);

drop table listings;
