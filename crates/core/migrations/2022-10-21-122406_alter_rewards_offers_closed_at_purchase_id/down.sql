alter table rewards_offers
  rename column closed_at to canceled_at;

alter table rewards_offers
  drop column purchase_id;

alter table rewards_offers
  add column is_initialized boolean;

alter table rewards_offers
  add column purchase_ticket varchar(48);

alter table reward_payouts
  add column purchase_ticket varchar(48) not null default md5(random()::text);

alter table reward_payouts
  drop constraint reward_payouts_pkey cascade,
  add primary key(purchase_ticket); 

alter table reward_payouts
  drop column purchase_id;