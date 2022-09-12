CREATE TABLE IF NOT EXISTS collections_volume
(
    collection text NOT NULL,
    _1d_volume numeric NOT NULL DEFAULT 0,
    _7d_volume numeric NOT NULL DEFAULT 0,
    _30d_volume numeric NOT NULL DEFAULT 0,
    _prev_1d_volume numeric NOT NULL DEFAULT 0,
    _prev_7d_volume numeric NOT NULL DEFAULT 0,
    _prev_30d_volume numeric NOT NULL DEFAULT 0,
    CONSTRAINT collections_volume_pkey PRIMARY KEY (collection)
);

INSERT INTO COLLECTIONS_VOLUME (COLLECTION)
    select distinct collection_address as collection from metadata_collection_keys;

INSERT INTO COLLECTIONS_VOLUME (COLLECTION)
    select distinct id::text as collection from me_collections;