create table metadata_jsons
(
    metadata_address varchar(48) primary key not null,
    fingerprint bytea,
    description text,
    image text,
    animation_url text,
    external_url text,
    category text,
    updated_at timestamp,
    raw_content jsonb
);

