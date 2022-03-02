create table store_creators (
    store_config_address varchar(48),
    creator_address      varchar(48),
    PRIMARY KEY(store_config_address,creator_address)
);

create index if not exists store_creators_config_addr_index
on store_creators (store_config_address);