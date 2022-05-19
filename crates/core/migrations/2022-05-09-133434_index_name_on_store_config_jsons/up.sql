create index if not exists name_store_config_jsons_idx on
  store_config_jsons using hash (name);