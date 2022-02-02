create table storefrontsv2_configs (
  address                         varchar(48)    PRIMARY KEY,
  settings_uri                    varchar(200)
);

create table storefrontsv2(
  store_address                     varchar(48)    PRIMARY KEY,
  public                            boolean,
  auction_program                   varchar(48),
  token_vault_program               varchar(48) ,
  token_metadata_program            varchar(48),
  token_program                     varchar(48),
  store_config_pda                  varchar(48)
);


create table whitelisted_creators (
  address                         varchar(48),
  creator_address                 varchar(48),
  activated                       boolean,
  PRIMARY KEY (address , creator_address)
);


create table storefrontsv2_whitelisted_creator_pdas(
  store_address                   varchar(48),
  creator_address                 varchar(48),
  PRIMARY KEY (store_address,creator_address)

);

create table settings_uri_jsons(
    store_config_pda varchar(48) PRIMARY KEY,
    name text,
    description text,
    logo_url text,
    banner_url text,
    subdomain text,
    owner_address varchar(48),
    auction_house_address varchar(48)
);

create index on storefrontsv2_configs (address);
create index on storefrontsv2 (store_address);
create index on whitelisted_creators (address,creator_address);
