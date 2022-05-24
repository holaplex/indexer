alter table offer_events     drop constraint offer_events_bid_receipt_address_fkey;
alter table purchase_events  drop constraint purchase_events_purchase_receipt_address_fkey;
alter table listing_events   drop constraint listing_events_listing_receipt_address_fkey;
