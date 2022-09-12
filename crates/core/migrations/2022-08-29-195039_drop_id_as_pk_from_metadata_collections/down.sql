alter table metadata_collections
drop constraint metadata_collections_pkey,
add column id uuid default gen_random_uuid(),
add primary key (id);