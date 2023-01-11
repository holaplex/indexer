DO $$
BEGIN
IF EXISTS ( 
	SELECT conrelid::regclass AS table_name, 
		   conname AS primary_key, 
		   pg_get_constraintdef(oid) 
	FROM   pg_constraint 
	WHERE  contype = 'p' 
	AND    connamespace = 'public'::regnamespace  
	AND    conrelid::regclass::text = 'metadata_collection_keys'
	and pg_get_constraintdef(oid) = 'PRIMARY KEY (metadata_address, collection_address)'
	ORDER  BY conrelid::regclass::text, contype DESC
) F THEN
ALTER TABLE metadata_collection_keys DROP CONSTRAINT metadata_collection_keys_pkey;
ALTER TABLE metadata_collection_keys ADD PRIMARY KEY (metadata_address);
END IF;
END $$;