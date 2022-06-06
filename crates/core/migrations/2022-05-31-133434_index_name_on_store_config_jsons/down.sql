alter table store_config_jsons
drop constraint uniq_subdomain;

drop index if exists subdomain_store_config_jsons_idx;