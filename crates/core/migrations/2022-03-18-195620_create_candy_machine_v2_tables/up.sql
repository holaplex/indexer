create table candy_machines (
    address                      varchar(48)                        primary key,
    authority                    varchar(48)                        not null,
    wallet                       varchar(48)                        not null,
    token_mint                   varchar(48),
    items_redeemed               bigint                             not null
);

create table candy_machine_datas (
    candy_machine_address        varchar(48)                        primary key,
    uuid                         text                               not null,
    price                        bigint                             not null,
    symbol                       text                               not null,
    seller_fee_basis_points      smallint                           not null,
    max_supply                   bigint                             not null,
    is_mutable                   bool                               not null,
    retain_authority             bool                               not null,
    go_live_date                 bigint,
    items_available              bigint                             not null
);

create table candy_machine_config_lines (
    address                      varchar(48)                        primary key,
    name                         text                               not null,
    uri                          text                               not null
);

create table candy_machine_creators (
    candy_machine_address        varchar(48)                        primary key,
    creator_address              varchar(48)                        not null,
    verified                     bool                               not null,
    share                        smallint                           not null
);

create table candy_machine_collection_pdas (
    address                      varchar(48)                        primary key,
    mint                         varchar(48)                        not null,
    candy_machine                varchar(48)                        not null
);

create table candy_machine_hidden_settings (
    candy_machine_address        varchar(48)                        primary key,
    name                         text                               not null,
    uri                          text                               not null,
    hash                         bytea                              not null
);

create table candy_machine_gate_keeper_configs (
    candy_machine_address        varchar(48)                        primary key,
    gatekeeper_network           varchar(48)                        not null,
    expire_on_use                bool                               not null
);

create type settingtype as enum ('Date', 'Amount');

create table candy_machine_end_settings (
    candy_machine_address        varchar(48)                        primary key,
    end_setting_type             settingtype                        not null,
    number                       bigint                             not null
);

create type mode as enum ('BurnEveryTime', 'NeverBurn');

create table candy_machine_whitelist_mint_settings (
    candy_machine_address        varchar(48)                        primary key,
    mode                         mode                               not null,
    mint                         varchar(48)                        not null,
    presale                      bool                               not null,
    discount_price               bigint
);