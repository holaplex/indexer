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
  inner join auction_houses
      on (listings.auction_house = auction_houses.address)
  left join collection_stats on (
    collection_stats.collection_address =
        metadata_collection_keys.collection_address
  )
  where metadatas.address = new.metadata
      and auction_houses.treasury_mint =
          'So11111111111111111111111111111111111111112'
      and listings.purchase_id is null
      and listings.canceled_at is null
      and metadata_collection_keys.verified = true
  group by metadata_collection_keys.collection_address

  on conflict (collection_address)
  do update set floor_price = excluded.floor_price;

  return null;
end
$$;

create trigger listing_added
after insert on listings for each row
execute procedure floor_price_on_listing_update();

create trigger listing_updated
after update on listings for each row
execute procedure floor_price_on_listing_update();
