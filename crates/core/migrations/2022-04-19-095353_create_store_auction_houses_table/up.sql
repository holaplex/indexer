create table store_auction_houses (
    store_config_address varchar(48) not null,
    auction_house_address varchar(48) not null,
    primary key (store_config_address, auction_house_address)
);

create index if not exists store_creators_config_addr_index
on store_auction_houses (store_config_address);
