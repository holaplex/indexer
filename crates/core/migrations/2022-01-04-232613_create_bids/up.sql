create table bids (
  listing_address  varchar(48) not null,
  bidder_address   varchar(48) not null,
  last_bid_time    timestamp   not null,
  last_bid_amount  bigint      not null,
  cancelled        boolean     not null,

  primary key (listing_address, bidder_address),
  foreign key (listing_address) references listings (address)
);

