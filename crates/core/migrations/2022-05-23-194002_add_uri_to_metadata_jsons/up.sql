alter table metadata_jsons
add column fetch_uri text not null default '';

alter table metadata_jsons
alter column fetch_uri drop default;
