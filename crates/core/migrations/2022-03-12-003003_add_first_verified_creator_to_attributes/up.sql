alter table attributes
add column first_verified_creator varchar(48) null;

create index if not exists attributes_first_verified_creator_idx
on attributes (first_verified_creator);