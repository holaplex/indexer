create table mint_events (
  metadata_address varchar(48) not null unique,
  feed_event_id uuid not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (metadata_address) references metadatas (address)
);

create index if not exists mint_events_metadata_address_idx on 
  mint_events using hash (metadata_address);
