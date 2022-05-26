alter table store_config_jsons
add constraint uniq_subdomain unique (subdomain);

create index if not exists name_store_config_jsons_idx on
  store_config_jsons using btree (name);