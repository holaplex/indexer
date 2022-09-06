do $$
begin
  	if (select not exists (select 1 from information_schema.tables  where table_schema = 'public' and table_name = 'me_collections')) then

	create table temp_me_collections as
	(select distinct m.symbol,
			mc.creator_address,
			t.name,
			t.family
		from metadatas m
		inner join metadata_creators mc on mc.metadata_address = m.address
		inner join purchases p on p.metadata = m.address
		left join metadata_collections t on t.metadata_address = m.address
		where p.marketplace_program = 'm2mx93ekt1fmxsvktrul9xvfhkmme8htui5cyc5af7k'
			and mc.verified = true
			and mc.position = 0
		group by (m.symbol, mc.creator_address, t.name, t.family)
		having count(*) >= 100);

	create table temp_me_collections_2 as (
		select symbol, name, family  
			from
				(select symbol, name, family
					from temp_me_collections
					where name is not null
						and family is not null
					group by (symbol, name, family)
					union all select symbol, name, family
					from temp_me_collections
					where name is null
						and family is null) as c);

	alter table temp_me_collections_2 add column id uuid default gen_random_uuid() primary key;

	alter table temp_me_collections add column id uuid;

	update temp_me_collections
	set id = v2.id
	from temp_me_collections v1
	inner join temp_me_collections_2 v2 on v1.symbol = v2.symbol
	and v1.name is not distinct
	from v2.name
	and v1.family is not distinct
	from v2.family
	where v1.family is not null
		and v1.name is not null
		and temp_me_collections.symbol = v1.symbol
		and temp_me_collections.name is not distinct
		from v1.name
		and temp_me_collections.family is not distinct
		from v1.family;

	create temp table distinct_me_ids as
		(select distinct on (v1.symbol, v1.creator_address, v2.symbol, v2.id) v1.symbol,
				v1.creator_address,
				v2.id
			from temp_me_collections v1
			inner join temp_me_collections_2 v2 on v1.symbol = v2.symbol
			where v1.family is null
				and v1.name is null
				and v2.family is null
				and v2.name is null );

	update temp_me_collections
	set id = v2.id
	from temp_me_collections v1
	inner join distinct_me_ids v2 on v1.symbol = v2.symbol
	and v1.creator_address = v2.creator_address
	where v1.family is null
		and v1.name is null
		and temp_me_collections.symbol = v1.symbol
		and temp_me_collections.name is null
		and temp_me_collections.family is null
		and temp_me_collections.creator_address = v1.creator_address;

	drop table distinct_me_ids;

	create table temp_me_metadata_collections as
		(select m.address as metadata_address,
				c.id as collection_id
			from temp_me_collections c
			inner join temp_me_collections cc on c.id = cc.id
			inner join metadata_creators mc on mc.creator_address = c.creator_address
			inner join metadatas m on m.address = mc.metadata_address
			and m.symbol = cc.symbol
			inner join metadata_collections tc on tc.metadata_address = m.address
			and tc.name is not distinct
			from cc.name
			and tc.family is not distinct
			from cc.family);

	delete
	from temp_me_metadata_collections
	where metadata_address in
			(select metadata_address
				from metadata_collection_keys);

	delete
	from temp_me_collections_2
	where id in
			(select mc.id
				from temp_me_collections_2 mc
				left join temp_me_metadata_collections t on mc.id = t.collection_id
				where t.collection_id is null);

	delete
	from temp_me_collections
	where id in
			(select mc.id
				from temp_me_collections mc
				left join temp_me_metadata_collections t on mc.id = t.collection_id
				where t.collection_id is null);

	begin

	lock table me_metadata_collections in access exclusive mode;

	drop table if exists me_metadata_collections;

	create table me_metadata_collections as
		(select distinct *
			from temp_me_metadata_collections);

	alter table me_metadata_collections add primary key (metadata_address, collection_id);

	end;

	drop table temp_me_collections;

	begin 

	lock table me_metadata_collections in access exclusive mode;

	drop table if exists me_collections;

	alter table temp_me_collections_2 rename to me_collections;

	end;

	drop table temp_me_metadata_collections;
	-- add image col to me_collections table

	alter table me_collections add column image text not null default '';

	update me_collections
	set image = m.image
	from
		(select c.id as collection_id,
				mj.image as image
			from me_collections c
			inner join me_metadata_collections mc1 on mc1.metadata_address =
				(select metadata_address
					from me_metadata_collections mc2
					where c.id = mc2.collection_id
					limit 1)
			inner join metadata_jsons mj on mj.metadata_address = mc1.metadata_address) m
	where me_collections.id = m.collection_id;

	-- me_collection_stats

	create table if not exists me_collection_stats (
	  collection_id       uuid 		  primary key,
	  nft_count           bigint      not null,
	  floor_price         bigint      null
	);

	insert into me_collection_stats (collection_id, nft_count, floor_price) with nft_count_table as
	(select me_metadata_collections.collection_id as collection_id,
			count(me_metadata_collections.metadata_address) as nft_count
		from me_metadata_collections
		group by me_metadata_collections.collection_id),
	floor_price_table as
	(select me_metadata_collections.collection_id as collection_id,
			min(listings.price) as floor_price
		from listings
		inner join metadatas on (listings.metadata = metadatas.address)
		inner join me_metadata_collections on (metadatas.address = me_metadata_collections.metadata_address)
		where listings.marketplace_program = 'm2mx93ekt1fmxsvktrul9xvfhkmme8htui5cyc5af7k'
			and listings.purchase_id is null
			and listings.canceled_at is null
		group by me_metadata_collections.collection_id)
	select nft_count_table.collection_id,
		nft_count_table.nft_count,
		floor_price_table.floor_price
	from nft_count_table,
		floor_price_table
	where nft_count_table.collection_id = floor_price_table.collection_id;
	end if;
end $$;