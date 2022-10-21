alter table rewards_offers
  rename column closed_at to canceled_at;

alter table rewards_offers
  drop column purchase_id;

alter table rewards_offers
  add column is_initialized boolean;

alter table rewards_offers
  add column purchase_ticket varchar(48);