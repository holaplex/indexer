create index if not exists purchase_receipts_buyer_idx on 
  purchase_receipts using hash (buyer);

create index if not exists purchase_receipts_seller_idx on 
  purchase_receipts using hash (seller);