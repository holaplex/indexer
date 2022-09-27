-- update 30d_marketcap for mcc
WITH marketcaps AS (
    SELECT
        metadata_collection_keys.collection_address AS collection,
        MIN(listings.price)::numeric * count(metadata_collection_keys.metadata_address)::numeric as marketcap
    FROM
        listings
        INNER JOIN metadata_collection_keys ON (listings.metadata = metadata_collection_keys.metadata_address)
            WHERE listings.purchase_id IS NULL
            AND listings.canceled_at IS NULL
            AND listings.CREATED_AT <= (NOW() - INTERVAL '1 months')
            AND listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
            GROUP BY metadata_collection_keys.collection_address)
UPDATE
    COLLECTION_TRENDS
SET
    _30D_MARKETCAP = m.marketcap
FROM
    marketcaps m
    INNER JOIN COLLECTION_TRENDS CT ON m.COLLECTION = CT.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CT.collection;

-- update 30d_marketcap for non mcc
WITH marketcaps AS (
    SELECT
        me_metadata_collections.collection_id::text AS collection,
        MIN(listings.price)::numeric * count(me_metadata_collections.metadata_address)::numeric as marketcap
    FROM
        listings
        INNER JOIN me_metadata_collections ON (listings.metadata = me_metadata_collections.metadata_address)
            WHERE listings.purchase_id IS NULL
            AND listings.canceled_at IS NULL
            AND listings.CREATED_AT <= (NOW() - INTERVAL '1 months')
            AND listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
            GROUP BY me_metadata_collections.collection_id)
UPDATE
    COLLECTION_TRENDS
SET
    _30D_MARKETCAP = m.marketcap
FROM
    marketcaps m
    INNER JOIN COLLECTION_TRENDS CT ON m.COLLECTION = CT.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CT.collection;

-- update prev_30d_marketcap for mcc
WITH marketcaps AS (
    SELECT
        metadata_collection_keys.collection_address AS collection,
        MIN(listings.price)::numeric * count(metadata_collection_keys.metadata_address)::numeric as marketcap
    FROM
        listings
        INNER JOIN metadata_collection_keys ON (listings.metadata = metadata_collection_keys.metadata_address)
            WHERE listings.purchase_id IS NULL
            AND listings.canceled_at IS NULL
            AND listings.CREATED_AT >= (NOW() - INTERVAL '2 months')
            AND listings.CREATED_AT <= (NOW() - INTERVAL '1 months')
            AND listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
            GROUP BY metadata_collection_keys.collection_address)
UPDATE
    COLLECTION_TRENDS
SET
    PREV_30D_MARKETCAP = m.marketcap
FROM
    marketcaps m
    INNER JOIN COLLECTION_TRENDS CT ON m.COLLECTION = CT.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CT.collection;

-- update prev_30d_marketcap for non mcc
WITH marketcaps AS (
    SELECT
        me_metadata_collections.collection_id::text AS collection,
        MIN(listings.price)::numeric * count(me_metadata_collections.metadata_address)::numeric as marketcap
    FROM
        listings
        INNER JOIN me_metadata_collections ON (listings.metadata = me_metadata_collections.metadata_address)
            WHERE listings.purchase_id IS NULL
            AND listings.canceled_at IS NULL
            AND listings.CREATED_AT >= (NOW() - INTERVAL '2 months')
            AND listings.CREATED_AT <= (NOW() - INTERVAL '1 months')
            AND listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
            GROUP BY me_metadata_collections.collection_id)
UPDATE
    COLLECTION_TRENDS
SET
    PREV_30D_MARKETCAP = m.marketcap
FROM
    marketcaps m
    INNER JOIN COLLECTION_TRENDS CT ON m.COLLECTION = CT.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CT.collection;