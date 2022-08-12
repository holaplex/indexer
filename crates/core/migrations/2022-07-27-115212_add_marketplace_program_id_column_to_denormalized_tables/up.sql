alter table offers
add column if not exists marketplace_program varchar(48) not null default '';

alter table listings
add column if not exists marketplace_program varchar(48) not null default '';

alter table purchases
add column if not exists marketplace_program varchar(48) not null default '';
