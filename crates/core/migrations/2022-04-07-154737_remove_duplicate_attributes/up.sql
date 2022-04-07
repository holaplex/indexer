create table temp as
    select distinct on (metadata_address, value, trait_type) * from attributes;

alter table attributes
    drop constraint if exists attributes_unique_constraint,
    drop constraint if exists attributes_primary_key_constraint;

alter table temp
    add constraint attributes_unique_constraint unique (metadata_address, value, trait_type),
    add constraint attributes_primary_key_constraint primary key (id),
    alter column metadata_address set not null,
    alter column id set default gen_random_uuid();

drop table attributes;
alter table temp rename to attributes;