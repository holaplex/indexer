delete from store_config_jsons 
  where store_address='3doAaFs2VuTLnVTPLZwFAWsskqwwC4xLt31dZ24uwYsd';

alter table store_config_jsons
add constraint uniq_subdomain unique (subdomain);

create index if not exists subdomain_store_config_jsons_idx on
  store_config_jsons using btree (subdomain);