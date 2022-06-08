insert into store_auction_houses (store_config_address, auction_house_address)
  select (config_address as store_config_address, auction_house_address) from store_config_jsons;