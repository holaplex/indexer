do $$
begin
  	if (select exists(select * from pg_catalog.pg_constraint where conname = 'metadata_collections_pkey'
					  and (conkey = array[4]::smallint[] or conkey = array[7]::smallint[] ))) then
		raise notice 'metadata_address as primary key for metadata_collections';
		raise notice 'acquring access exclusive lock';
		begin
		lock table metadata_collections in access exclusive mode;
		
		create table t_metadata_collections_slot_desc as
			(select *
				from metadata_collections
				order by slot desc);
				
		raise notice 'created t_metadata_collections_slot_desc';
		
		create table t_metadata_collections as
	        (select distinct on (metadata_address) *
		    from
			t_metadata_collections_slot_desc a);
		
		raise notice 'created t_metadata_collections';

		drop table metadata_collections;

		raise notice 'deleted';
		
		alter table t_metadata_collections
        drop column id;

		alter table t_metadata_collections rename to metadata_collections;
		
		alter table metadata_collections
        add constraint metadata_collections_pkey primary key (metadata_address),
		alter column slot set not null,
		alter column write_version set not null;

		create index coll_name_metadata_address_idx on metadata_collections using btree (name, metadata_address);
		create index metadata_collections_slot_idx on metadata_collections using btree (slot);
		
		create trigger metadata_collections_check_slot_wv
		before
		update on metadata_collections
		for row execute function check_slot_wv();

		end;
		
		raise notice 'lock released';
		
		drop table t_metadata_collections_slot_desc;
	end if;

end $$;

