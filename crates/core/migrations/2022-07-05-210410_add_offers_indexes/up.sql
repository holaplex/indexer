create index if not exists offers_canceled_at_purchase_id_null_idx on
  offers (canceled_at, purchase_id) WHERE canceled_at IS NULL AND purchase_id IS NULL;

create index if not exists offers_auction_house_idx on
  offers using hash (auction_house);

create index if not exists offers_metadata_idx on
  offers using hash (metadata);

create index if not exists offers_buyer_idx on
  offers using hash (buyer);
