alter table attributes 
drop constraint if exists attributes_unique_constraint;

alter table attributes
drop constraint if exists attributes_primary_key_constraint;

drop table attributes;

alter table temp_attributes rename to attributes;