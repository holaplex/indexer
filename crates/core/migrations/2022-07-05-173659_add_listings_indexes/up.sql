create index if not exists listings_canceled_at_purchase_id_null_idx on
  listings (canceled_at, purchase_id) WHERE canceled_at IS NULL AND purchase_id IS NULL;

create index if not exists listings_auction_house_idx on
  listings using hash (auction_house);

create index if not exists listings_metadata_idx on
  listings using hash (metadata);

create index if not exists listings_seller_idx on
  listings using hash (seller);
