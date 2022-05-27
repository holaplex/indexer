alter table files
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;

alter table attributes
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;

alter table metadata_collections
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;

alter table metadata_jsons
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;

alter table files
alter column slot          drop default,
alter column write_version drop default;

alter table attributes
alter column slot          drop default,
alter column write_version drop default;

alter table metadata_collections
alter column slot          drop default,
alter column write_version drop default;

alter table metadata_jsons
alter column slot          drop default,
alter column write_version drop default;
