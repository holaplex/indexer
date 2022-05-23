alter table files
drop column slot,
drop column write_version;

alter table attributes
drop column slot,
drop column write_version;

alter table metadata_collections
drop column slot,
drop column write_version;

alter table metadata_jsons
drop column slot,
drop column write_version;
