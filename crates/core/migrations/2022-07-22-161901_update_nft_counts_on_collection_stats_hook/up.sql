create or replace function nft_count_on_nft_added() returns trigger
  language plpgsql
  as $$
begin
  insert into collection_stats (collection_address, nft_count)
    values (new.collection_address, 1)
    on conflict (collection_address) do update
      set nft_count = collection_stats.nft_count + 1;

  return null;
end
$$;

create trigger nft_collection_key_added
after insert on metadata_collection_keys for each row
execute procedure nft_count_on_nft_added();
