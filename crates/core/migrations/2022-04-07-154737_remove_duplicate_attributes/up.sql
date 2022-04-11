create function create_temp_attributes_table()
  returns void
  language plpgsql as
$func$
begin
   if not exists (select from pg_catalog.pg_tables 
              where  schemaname = 'public'
              and    tablename  = 'temp_attributes') then
              
     	alter table attributes rename to temp_attributes;

      create table attributes (
		  metadata_address      varchar(48)     not null,
		  value                 text,
		  trait_type            text,
		  id                    uuid            primary key default gen_random_uuid(),
		  first_verified_creator varchar(48)    null,
		  unique (metadata_address, value, trait_type)
		);

		create index if not exists attr_metadata_address_index on 
		attributes using hash (metadata_address);

		create index if not exists attr_first_verified_creator_index
		on attributes (first_verified_creator);
		
    create index if not exists attr_trait_type_value_index
		on attributes (trait_type, value);

   end if;
end
$func$;

select create_temp_attributes_table();