alter table metadatas
add column slot bigint null;

alter table editions
add column slot bigint null;

alter table master_editions
add column slot bigint null;