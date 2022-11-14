create table attribute_groups (
    collection_id   text    not null, 
    trait_type      text    not null,
    total_count     bigint  not null,
    primary key (collection_id, trait_type)
);

create table attribute_group_variants (
    collection_id text not null,
    trait_type text not null,
    value text not null,
    count bigint not null,
    primary key (collection_id, trait_type, value)
);

