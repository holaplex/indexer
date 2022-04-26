create table follow_events (
  graph_connection_address varchar(48) not null unique,
  feed_event_id uuid not null,
  primary key (feed_event_id),
  foreign key (feed_event_id) references feed_events (id),
  foreign key (graph_connection_address) references graph_connections (address)
);

create index if not exists follow_events_graph_connection_address_idx on 
  follow_events using hash (graph_connection_address);