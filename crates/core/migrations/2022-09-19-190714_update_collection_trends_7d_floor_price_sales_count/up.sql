--update prev_7d_floor_price for mcc
WITH floor_price_table AS (
    SELECT
        metadata_collection_keys.collection_address AS collection,
        min(listings.price) AS floor_price
    FROM
        listings
        INNER JOIN metadata_collection_keys ON (listings.metadata = metadata_collection_keys.metadata_address)
            AND listings.purchase_id IS NULL
            AND listings.canceled_at IS NULL
            AND metadata_collection_keys.verified = TRUE
            AND listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
    WHERE
        listings.CREATED_AT <= (NOW() - INTERVAL '1 weeks')
    GROUP BY
        metadata_collection_keys.collection_address)
UPDATE
    COLLECTION_TRENDS
SET
    PREV_7D_FLOOR_PRICE = f.floor_price
FROM
    floor_price_table f
    INNER JOIN COLLECTION_TRENDS CV ON f.COLLECTION = CV.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CV.collection;

-- update _prev_7d_floor_price for non mcc
WITH floor_price_table AS (
    SELECT
        me_metadata_collections.collection_id::text AS collection,
        min(listings.price) AS floor_price
    FROM
        listings
        INNER JOIN me_metadata_collections ON (listings.metadata = me_metadata_collections.metadata_address)
            AND listings.purchase_id IS NULL
            AND listings.canceled_at IS NULL
            AND listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
    WHERE
        listings.CREATED_AT <= (NOW() - INTERVAL '1 weeks')
    GROUP BY
        me_metadata_collections.collection_id)
UPDATE
    COLLECTION_TRENDS
SET
    PREV_7D_FLOOR_PRICE = f.floor_price
FROM
    floor_price_table f
    INNER JOIN COLLECTION_TRENDS CV ON f.COLLECTION = CV.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CV.collection;

-- update _7d_sales_count for mcc
WITH sales_count AS (
    SELECT
        metadata_collection_keys.collection_address AS collection,
        count(purchases.id) AS sales_count
    FROM
        purchases
        INNER JOIN metadata_collection_keys ON (purchases.metadata = metadata_collection_keys.metadata_address)
    WHERE
        metadata_collection_keys.verified = TRUE
        AND purchases.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
        AND purchases.CREATED_AT >= (NOW() - INTERVAL '1 weeks')
    GROUP BY
        metadata_collection_keys.collection_address)
UPDATE
    COLLECTION_TRENDS
SET
    _7D_SALES_COUNT = s.sales_count
FROM
    sales_count s
    INNER JOIN COLLECTION_TRENDS CV ON s.COLLECTION = CV.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CV.collection;

-- update prev_7d_sales_count for mcc
WITH sales_count AS (
    SELECT
        metadata_collection_keys.collection_address AS collection,
        count(purchases.id) AS sales_count
    FROM
        purchases
        INNER JOIN metadata_collection_keys ON (purchases.metadata = metadata_collection_keys.metadata_address)
    WHERE
        metadata_collection_keys.verified = TRUE
        AND purchases.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
        AND purchases.CREATED_AT >= (NOW() - INTERVAL '2 weeks')
        AND purchases.CREATED_AT <= (NOW() - INTERVAL '1 weeks')
    GROUP BY
        metadata_collection_keys.collection_address)
UPDATE
    COLLECTION_TRENDS
SET
    PREV_7D_SALES_COUNT = s.sales_count
FROM
    sales_count s
    INNER JOIN COLLECTION_TRENDS CV ON s.COLLECTION = CV.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CV.collection;

-- update _7d_sales_count for non_mcc
WITH sales_count AS (
    SELECT
        me_metadata_collections.collection_id::text AS collection,
        count(purchases.id) AS sales_count
    FROM
        purchases
        INNER JOIN me_metadata_collections ON (purchases.metadata = me_metadata_collections.metadata_address)
    WHERE
        purchases.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
        AND purchases.CREATED_AT >= (NOW() - INTERVAL '1 weeks')
    GROUP BY
        me_metadata_collections.collection_id)
UPDATE
    COLLECTION_TRENDS
SET
    _7D_SALES_COUNT = s.sales_count
FROM
    sales_count s
    INNER JOIN COLLECTION_TRENDS CV ON s.COLLECTION = CV.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CV.collection;

-- update prev_7d_sales_count for non_mcc
WITH sales_count AS (
    SELECT
        me_metadata_collections.collection_id::text AS collection,
        count(purchases.id) AS sales_count
    FROM
        purchases
        INNER JOIN me_metadata_collections ON (purchases.metadata = me_metadata_collections.metadata_address)
    WHERE
        purchases.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
        AND purchases.CREATED_AT >= (NOW() - INTERVAL '2 weeks')
        AND purchases.CREATED_AT <= (NOW() - INTERVAL '1 weeks')
    GROUP BY
        me_metadata_collections.collection_id)
UPDATE
    COLLECTION_TRENDS
SET
    PREV_7D_SALES_COUNT = s.sales_count
FROM
    sales_count s
    INNER JOIN COLLECTION_TRENDS CV ON s.COLLECTION = CV.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CV.collection;