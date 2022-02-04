create table storefrontsv2_configs (
  address                         varchar(48)    PRIMARY KEY,
  settings_uri                    varchar(200)   not null 
);

create table storefrontsv2(
  store_address                     varchar(48)    PRIMARY KEY,
  public                            boolean        not null,
  auction_program                   varchar(48)    not null,
  token_vault_program               varchar(48)    not null,
  token_metadata_program            varchar(48)    not null,
  token_program                     varchar(48)    not null,
  store_config_pda                  varchar(48)    not null
);


create table whitelisted_creators (
  address                         varchar(48),
  creator_address                 varchar(48),
  activated                       boolean,
  PRIMARY KEY (address , creator_address)
);


-- create table storefrontsv2_whitelisted_creator_pdas(
--   store_address                   varchar(48),
--   creator_address                 varchar(48),
--   PRIMARY KEY (store_address,creator_address)

-- );

create table settings_uri_jsons(
    store_config_pda varchar(48)      PRIMARY KEY,
    name text                         not null,
    description text                  not null,
    logo_url text                     not null,
    banner_url text                   not null,
    subdomain text                    not null,
    owner_address varchar(48)         not null,
    auction_house_address varchar(48) not null
);

create index on storefrontsv2_configs (address);
create index on storefrontsv2 (store_address);
create index on whitelisted_creators (creator_address);
