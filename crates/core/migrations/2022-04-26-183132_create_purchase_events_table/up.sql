create table purchase_events (
  purchase_receipt_address varchar(48) not null unique,
  feed_event_id uuid not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (purchase_receipt_address) references purchase_receipts (address)
);

create index if not exists purchase_events_purchase_receipt_address_idx on 
  purchase_events using hash (purchase_receipt_address);