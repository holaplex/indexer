drop table mint_events;
drop table feed_event_wallets;
drop table feed_events;
drop index if exists feed_events_created_at_desc_idx;
drop index if exists mint_events_feed_event_id_idx;
drop index if exists feed_event_wallets_feed_event_id_idx;
drop index if exists mint_events_metadata_address_idx;