create table twitter_handle_name_services (
    address                      varchar(48)    primary key,
    wallet_address               varchar(48)    not null,
    twitter_handle               text           not null,
    slot                         bigint         not null                      
);

create index if not exists twitter_handle_name_services_wallet_idx
on twitter_handle_name_services (wallet_address);

create index if not exists twitter_handle_name_services_handle_idx
on twitter_handle_name_services (twitter_handle);