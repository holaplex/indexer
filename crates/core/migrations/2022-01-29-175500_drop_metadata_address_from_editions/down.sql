delete from editions;
delete from master_editions;

alter table editions
add column metadata_address varchar(48) not null;

alter table master_editions
add column metadata_address varchar(48) not null;