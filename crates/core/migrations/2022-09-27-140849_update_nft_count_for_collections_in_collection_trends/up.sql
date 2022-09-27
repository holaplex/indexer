with
    nft_count_table as (
        select
            metadata_collection_keys.collection_address as collection,
            count(metadata_collection_keys.metadata_address) as nft_count
        from metadata_collection_keys
        where metadata_collection_keys.verified = true
        group by metadata_collection_keys.collection_address
        union all
        select
            me_metadata_collections.collection_id::text as collection,
            count(me_metadata_collections.metadata_address) as nft_count
        from me_metadata_collections
        group by me_metadata_collections.collection_id
    )
    UPDATE
    COLLECTION_TRENDS
SET
    NFT_COUNT = n.nft_count
FROM
    nft_count_table n
    INNER JOIN COLLECTION_TRENDS CT ON n.COLLECTION = CT.COLLECTION
WHERE
    COLLECTION_TRENDS.collection = CT.collection;