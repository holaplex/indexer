drop table metadata_collection_keys;
alter table attributes drop column token_standard;
drop type if exists token_standard;
