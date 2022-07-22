INSERT INTO collection_stats (collection_address, nft_count, floor_price)
WITH nft_count_table as (
    SELECT metadata_collection_keys.collection_address AS collection_address, COUNT (metadata_collection_keys.metadata_address) AS nft_count
    FROM metadata_collection_keys
    WHERE metadata_collection_keys.verified = true
    GROUP BY metadata_collection_keys.collection_address
),

floor_price_table as (
    SELECT metadata_collection_keys.collection_address AS collection_address, MIN(listings.price) AS floor_price
    FROM listings
    INNER JOIN metadatas ON(listings.metadata = metadatas.address)
    INNER JOIN metadata_collection_keys ON(metadatas.address = metadata_collection_keys.metadata_address)
    INNER JOIN auction_houses ON(listings.auction_house = auction_houses.address)
    WHERE
        auction_houses.treasury_mint = 'So11111111111111111111111111111111111111112'
        AND listings.purchase_id IS NULL
        AND listings.canceled_at IS NULL
        AND metadata_collection_keys.verified = true
    GROUP BY metadata_collection_keys.collection_address

)

SELECT nft_count_table.collection_address, nft_count_table.nft_count, floor_price_table.floor_price
FROM nft_count_table, floor_price_table
WHERE nft_count_table.collection_address = floor_price_table.collection_address