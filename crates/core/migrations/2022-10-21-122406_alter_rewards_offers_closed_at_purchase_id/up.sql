alter table rewards_offers
  rename column canceled_at to closed_at;

alter table rewards_offers
  drop column purchase_ticket;

alter table rewards_offers
  drop column is_initialized;

alter table rewards_offers
  add column purchase_id uuid;