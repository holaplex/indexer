alter table attributes rename to temp_attributes;

create table attributes (
  metadata_address      varchar(48)     not null,
  value                 text,
  trait_type            text,
  id                    uuid            primary key default gen_random_uuid(),
  first_verified_creator varchar(48)    null,
  unique (metadata_address, value, trait_type)
);

create index if not exists attributes_metadata_address_idx on 
attributes using hash (metadata_address);

create index if not exists attributes_first_verified_creator_idx
on attributes (first_verified_creator);