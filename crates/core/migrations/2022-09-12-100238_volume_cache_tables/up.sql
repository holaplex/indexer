CREATE TABLE IF NOT EXISTS collections_volume
(
    collection text NOT NULL,
    _1d_volume numeric,
    _7d_volume numeric,
    _30d_volume numeric,
    _prev_1d_volume numeric,
    _prev_7d_volume numeric,
    _prev_30d_volume numeric,
    CONSTRAINT collections_volume_pkey PRIMARY KEY (collection)
);

INSERT INTO COLLECTIONS_VOLUME (COLLECTION)
    select distinct collection_address as collection from metadata_collection_keys on conflict do nothing;

INSERT INTO COLLECTIONS_VOLUME (COLLECTION)
    select distinct id::text as collection from me_collections on conflict do nothing;