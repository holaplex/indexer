create type listingeventlifecycle as ENUM('Created', 'Cancelled');

create table listing_events (
  listing_receipt_address varchar(48) not null,
  feed_event_id uuid not null,
  lifecycle listingeventlifecycle not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (listing_receipt_address) references listing_receipts (address),
  constraint uc_listing_events_listing_receipt_address_lifecycle UNIQUE (listing_receipt_address, lifecycle)
);

create index if not exists listing_events_listing_receipt_address_idx on 
  listing_events using hash (listing_receipt_address);