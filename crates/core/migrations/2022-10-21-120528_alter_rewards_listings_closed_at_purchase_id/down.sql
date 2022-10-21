alter table rewards_listings
  rename column closed_at to canceled_at;

alter table rewards_listings
  drop column purchase_id;

alter table rewards_listings
  add column is_initialized boolean;

alter table rewards_listings
  add column purchase_ticket varchar(48);