alter table store_config_jsons
drop constraint uniq_subdomain;

drop index if exists name_store_config_jsons_idx;