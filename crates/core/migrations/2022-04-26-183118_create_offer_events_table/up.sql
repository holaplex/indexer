create type offereventlifecycle as ENUM('Created', 'Cancelled');

create table offer_events (
  bid_receipt_address varchar(48) not null,
  feed_event_id uuid not null,
  lifecycle offereventlifecycle  not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (bid_receipt_address) references bid_receipts (address),
  constraint uc_offer_events_bid_receipt_address_lifecycle UNIQUE (bid_receipt_address, lifecycle)
);

create index if not exists offer_events_bid_receipt_address_idx on 
  offer_events using hash (bid_receipt_address);
