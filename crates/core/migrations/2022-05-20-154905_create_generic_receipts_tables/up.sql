create table offers (
    address                 uuid            primary key default gen_random_uuid(),
    trade_state             varchar(48)     not null,
    bookkeeper              varchar(48)     not null,
    auction_house           varchar(48)     not null,
    buyer                   varchar(48)     not null,
    metadata                varchar(48)     not null,
    token_account           varchar(48),
    purchase_receipt        varchar(48),
    price                   bigint          not null,
    token_size              bigint          not null,
    bump                    smallint,
    trade_state_bump        smallint        not null,
    created_at              timestamp       not null,
    canceled_at             timestamp,

    CONSTRAINT offers_unique_fields UNIQUE 
    (trade_state, bookkeeper, auction_house, buyer, metadata, price, token_size, trade_state_bump)
);

