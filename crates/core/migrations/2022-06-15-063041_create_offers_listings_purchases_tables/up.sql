create table offers (
    id                      uuid            primary key default gen_random_uuid(),
    trade_state             varchar(48)     not null,
    auction_house           varchar(48)     not null,
    buyer                   varchar(48)     not null,
    metadata                varchar(48)     not null,
    token_account           varchar(48),
    purchase_id             uuid,
    price                   bigint          not null,
    token_size              bigint          not null,
    trade_state_bump        smallint        not null,
    created_at              timestamp       not null,
    canceled_at             timestamp,
    slot                    bigint          not null default -1,
    write_version           bigint,

    constraint offers_unique_fields unique 
    (trade_state, auction_house, buyer, metadata, price, token_size, trade_state_bump)
);

create index if not exists offers_trade_state_idx on 
  offers using hash (trade_state);

create table purchases (
    id                 uuid            primary key default gen_random_uuid(),
    buyer                   varchar(48)     not null,
    seller                  varchar(48)     not null,
    auction_house           varchar(48)     not null,
    metadata                varchar(48)     not null,
    token_size              bigint          not null,
    price                   bigint          not null,
    created_at              timestamp       not null,
    slot                    bigint          not null default -1,
    write_version           bigint,

    constraint purchases_unique_fields unique 
    (buyer, seller, auction_house, metadata, token_size, price)
);

create table listings (
    id                      uuid            primary key default gen_random_uuid(),
    trade_state             varchar(48)     not null,
    auction_house           varchar(48)     not null,
    seller                  varchar(48)     not null,
    metadata                varchar(48)     not null,
    purchase_id             uuid,
    price                   bigint          not null,
    token_size              bigint          not null,
    trade_state_bump        smallint        not null,
    created_at              timestamp       not null,
    canceled_at             timestamp,
    slot                    bigint          not null default -1,
    write_version           bigint,

    constraint listings_unique_fields unique 
    (trade_state, auction_house, seller, metadata, price, token_size, trade_state_bump)
);

create index if not exists listings_trade_state_idx on 
  listings using hash (trade_state);

create trigger offers_check_slot_wv
before update on offers for row
execute function check_slot_wv();

create trigger listings_check_slot_wv
before update on listings for row
execute function check_slot_wv();

create trigger purchases_check_slot_wv
before update on purchases for row
execute function check_slot_wv();