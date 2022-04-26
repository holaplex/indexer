create table feed_events (
  id uuid primary key default gen_random_uuid(),
  created_at timestamp with time zone not null default now()
);

create index if not exists feed_events_created_at_desc_idx on
  feed_events (created_at desc);
