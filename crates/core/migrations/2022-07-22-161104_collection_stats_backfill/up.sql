create or replace function collection_stats_backfill() returns void
  language plpgsql
  as $$

BEGIN

IF NOT EXISTS (SELECT * FROM collection_stats) THEN
  insert into collection_stats (collection_address, nft_count, floor_price)
with
    nft_count_table as (
        select
            metadata_collection_keys.collection_address as collection_address,
            count(metadata_collection_keys.metadata_address) as nft_count
        from metadata_collection_keys
        where metadata_collection_keys.verified = true
        group by metadata_collection_keys.collection_address
    ),

    floor_price_table as (
        select
            metadata_collection_keys.collection_address as collection_address,
            min(listings.price) as floor_price
        from listings
        inner join metadatas on (listings.metadata = metadatas.address)
        inner join metadata_collection_keys
            on (metadatas.address = metadata_collection_keys.metadata_address)
        inner join auction_houses
            on (listings.auction_house = auction_houses.address)
        where
            auction_houses.treasury_mint = 'So11111111111111111111111111111111111111112'
            and listings.purchase_id is null
            and listings.canceled_at is null
            and metadata_collection_keys.verified = true
        group by metadata_collection_keys.collection_address
    )

select
    nft_count_table.collection_address,
    nft_count_table.nft_count,
    floor_price_table.floor_price
from nft_count_table, floor_price_table
where nft_count_table.collection_address = floor_price_table.collection_address;
END IF;

END $$;


SELECT collection_stats_backfill();