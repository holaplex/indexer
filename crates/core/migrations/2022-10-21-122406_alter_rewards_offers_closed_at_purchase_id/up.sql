alter table rewards_offers
  rename column canceled_at to closed_at;

alter table rewards_offers
  drop column purchase_ticket;

alter table rewards_offers
  drop column is_initialized;

alter table rewards_offers
  add column purchase_id uuid;

alter table reward_payouts
  add column purchase_id uuid default gen_random_uuid();

alter table reward_payouts
  drop constraint reward_payouts_pkey cascade,
  add primary key(purchase_id); 

alter table reward_payouts
  drop column purchase_ticket;