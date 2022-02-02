delete from storefronts;

alter table storefronts
drop constraint storefronts_pkey,
add column address varchar(48) primary key not null;