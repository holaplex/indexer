create index if not exists buyer_bid_receipts_idx on
  bid_receipts using hash (buyer);