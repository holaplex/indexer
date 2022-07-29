alter table offers
add column if not exists marketplace_program varchar(48);

alter table listings
add column if not exists marketplace_program varchar(48);

alter table purchases
add column if not exists marketplace_program varchar(48);
