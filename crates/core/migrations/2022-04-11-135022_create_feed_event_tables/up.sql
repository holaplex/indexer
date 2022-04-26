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
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (metadata_address) references metadatas (address)
);

create table offer_events (
  bid_receipt_address varchar(48) not null,
  feed_event_id uuid not null,
  lifecycle text check (lifecycle IN ('Created', 'Cancelled')) not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (bid_receipt_address) references bid_receipts (address)
);

create table listing_events (
  listing_receipt_address varchar(48) not null,
  feed_event_id uuid not null,
  lifecycle text check (lifecycle IN ('Created', 'Cancelled')) not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (listing_receipt_address) references listing_receipts (address)
);

create table purchase_events (
  purchase_receipt_address varchar(48) not null unique,
  feed_event_id uuid not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (purchase_receipt_address) references purchase_receipts (address)
);

create table follow_events (
  graph_connection_address varchar(48) not null unique,
  feed_event_id uuid not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (graph_connection_address) references graph_connections (address)
);

create index if not exists feed_events_created_at_desc_idx on
  feed_events (created_at desc);

create index if not exists mint_events_metadata_address_idx on 
  mint_events using hash (metadata_address);

create index if not exists feed_event_wallets_feed_event_id_idx on 
  feed_event_wallets using hash (feed_event_id);

create index if not exists feed_event_wallets_wallet_address_idx on
  feed_event_wallets using hash (wallet_address);

create index if not exists offer_events_bid_receipt_address_idx on 
  offer_events using hash (bid_receipt_address);

create index if not exists listing_events_listing_receipt_address_idx on 
  listing_events using hash (listing_receipt_address);

create index if not exists purchase_events_purchase_receipt_address_idx on 
  purchase_events using hash (purchase_receipt_address);

create index if not exists follow_events_graph_connection_address_idx on 
  follow_events using hash (graph_connection_address);