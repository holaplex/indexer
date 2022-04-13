create table feed_events (
  id uuid primary key default gen_random_uuid(),
  created_at timestamp with time zone not null default now()
);

create table feed_event_wallets (
  wallet_address varchar(48) not null,
  feed_event_id uuid not null,
  primary key (wallet_address, feed_event_id),
  foreign key (feed_event_id) references feed_events (id)
);

create table mint_events (
  metadata_address varchar(48) not null unique,
  feed_event_id uuid not null,
  primary key (metadata_address, feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (metadata_address) references metadatas (address)
);

create index if not exists feed_events_created_at_desc_idx on
  feed_events (created_at desc);

create index if not exists mint_events_feed_event_id_idx on 
  mint_events using hash (feed_event_id);

create index if not exists mint_events_metadata_address_idx on 
  mint_events using hash (metadata_address);

create index if not exists feed_event_wallets_feed_event_id_idx on 
  feed_event_wallets using hash (feed_event_id);

create index if not exists feed_event_wallets_wallet_address_idx on
  feed_event_wallets using hash (wallet_address);
