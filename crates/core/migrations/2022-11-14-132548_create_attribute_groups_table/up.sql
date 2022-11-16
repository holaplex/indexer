create table attribute_groups(
    collection_id text not null,
    trait_type text not null,
    value text not null,
    count bigint not null,
    primary key (collection_id, trait_type, value)
);

create index on attribute_groups (collection_id);