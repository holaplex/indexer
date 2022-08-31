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
            and listings.purchase_id is null
            and listings.canceled_at is null
            and metadata_collection_keys.verified = true
			and listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
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

create or replace function floor_price_on_listing_update() returns trigger
  language plpgsql
  as $$
begin
  insert into collection_stats (collection_address, nft_count, floor_price)
  select
      metadata_collection_keys.collection_address as collection_address,
      coalesce(max(collection_stats.nft_count), 0) as nft_count,
      min(listings.price) as floor_price
  from listings
  inner join metadatas on (listings.metadata = metadatas.address)
  inner join metadata_collection_keys
      on (metadatas.address = metadata_collection_keys.metadata_address)
  left join collection_stats on (
    collection_stats.collection_address =
        metadata_collection_keys.collection_address
  )
  where metadatas.address = new.metadata
      and listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
      and listings.purchase_id is null
      and listings.canceled_at is null
      and metadata_collection_keys.verified = true
  group by metadata_collection_keys.collection_address

  on conflict (collection_address)
  do update set floor_price = excluded.floor_price;

  return null;
end
$$;

SELECT collection_stats_backfill();