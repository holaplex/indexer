create index if not exists purchases_metadata_idx on
  purchases using hash (metadata);

create index if not exists purchases_created_at_idx on
  purchases (created_at);

create index if not exists purchases_auction_house_idx on
  purchases using hash (auction_house);

create index if not exists purchases_seller_idx on
  purchases using hash (seller);

create index if not exists purchases_buyer_idx on
  purchases using hash (buyer);