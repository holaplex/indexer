alter table editions
drop constraint editions_metadata_address_fkey,
drop column metadata_address;

alter table master_editions
drop constraint master_editions_metadata_address_fkey,
drop column metadata_address;
