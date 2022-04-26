create table feed_event_wallets (
  wallet_address varchar(48) not null,
  feed_event_id uuid not null,
  primary key (wallet_address, feed_event_id),
  foreign key (feed_event_id) references feed_events (id)
);

create index if not exists feed_event_wallets_feed_event_id_idx on 
  feed_event_wallets using hash (feed_event_id);

create index if not exists feed_event_wallets_wallet_address_idx on
  feed_event_wallets using hash (wallet_address);
