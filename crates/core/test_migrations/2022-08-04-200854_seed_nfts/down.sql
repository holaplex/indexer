delete from metadatas where address = any(array [
  'meta_address_0',
  'meta_address_1',
  'meta_address_2',
  'meta_address_3',
  'collection_address'
]);

delete from metadata_jsons where metadata_address = any(array [
  'meta_address_0',
  'meta_address_1',
  'meta_address_2',
  'meta_address_3',
  'collection_address'
]);

delete from metadata_collection_keys where collection_address = any(array [
  'collection_mint'
]);

delete from current_metadata_owners where mint_address = any(array [
  'mint_0',
  'mint_1',
  'mint_2',
  'mint_3',
  'collection_mint'
]);
