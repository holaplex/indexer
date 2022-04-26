alter table offer_events
  add constraint uc_offer_events_bid_receipt_address_lifecycle UNIQUE (bid_receipt_address, lifecycle);

alter table listing_events
  add constraint uc_listing_events_listing_receipt_address_lifecycle UNIQUE (listing_receipt_address, lifecycle);
