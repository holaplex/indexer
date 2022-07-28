alter table offers
add column expiry timestamp;

alter table listings
add column expiry timestamp;

create index if not exists listings_expiry_idx on listings(expiry);
create index if not exists offers_expiry_idx on offers(expiry);
