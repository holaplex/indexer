create table store_configs (
  address       varchar(48) primary key,
  settings_uri  text        not null 
);

create table stores (
  address         varchar(48) primary key,
  public          boolean     not null,
  config_address  varchar(48) not null
);

create table whitelisted_creators (
  address         varchar(48) primary key,
  creator_address varchar(48) not null,
  activated       boolean     not null
);

create index on whitelisted_creators (creator_address);

-- create table store_whitelisted_creators (
--   store_address   varchar(48),
--   creator_address varchar(48),

--   PRIMARY KEY (store_address,creator_address)
-- );

create table store_config_jsons (
    config_address        varchar(48) primary key,
    name                  text        not null,
    description           text        not null,
    logo_url              text        not null,
    banner_url            text        not null,
    -- ???
    subdomain             text        not null,
    owner_address         varchar(48) not null,
    auction_house_address varchar(48) not null
);