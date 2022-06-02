alter table offer_events 
add constraint offer_events_bid_receipt_address_fkey 
foreign key (bid_receipt_address) references bid_receipts(address);

alter table purchase_events
add constraint purchase_events_purchase_receipt_address_fkey 
foreign key (purchase_receipt_address) references purchase_receipts(address);

alter table listing_events 
add constraint listing_events_listing_receipt_address_fkey 
foreign key (listing_receipt_address) references listing_receipts(address);