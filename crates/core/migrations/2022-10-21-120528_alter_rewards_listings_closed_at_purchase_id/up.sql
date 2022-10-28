alter table rewards_listings
  rename column canceled_at to closed_at;

alter table rewards_listings
  drop column purchase_ticket;

alter table rewards_listings
  drop column is_initialized;

alter table rewards_listings
  add column purchase_id uuid;
