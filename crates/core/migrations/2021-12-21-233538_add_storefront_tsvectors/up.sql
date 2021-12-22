alter table storefronts
add column ts_index tsvector not null
  generated always as (
    setweight(to_tsvector('english', title), 'A') ||
      setweight(to_tsvector('english', description), 'B')
  ) stored;

create index storefronts_ts_index on storefronts using gin (ts_index);
