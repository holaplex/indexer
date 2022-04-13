create index if not exists seller_listing_receipts_idx on
  listing_receipts using hash (seller);