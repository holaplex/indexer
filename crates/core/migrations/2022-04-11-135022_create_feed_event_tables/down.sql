drop table offer_events;
drop table mint_events;
drop table listing_events;
drop table purchase_events;
drop table follow_events;
drop table feed_event_wallets;
drop table feed_events;

drop type if exists offereventlifecycle;
drop type if exists listingeventlifecycle;

drop index if exists feed_events_created_at_desc_idx;
drop index if exists feed_event_wallets_feed_event_id_idx;
drop index if exists feed_event_wallets_wallet_address_idx;
drop index if exists mint_events_metadata_address_idx;
drop index if exists offer_events_bid_receipt_address_idx;
drop index if exists listing_events_listing_receipt_address_idx;
drop index if exists purchase_events_purchase_receipt_address_idx;
drop index if exists follow_events_graph_connection_address_idx;