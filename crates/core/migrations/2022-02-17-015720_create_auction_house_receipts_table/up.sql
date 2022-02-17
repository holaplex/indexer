create table receipts (
    address             varchar(48)     primary key,
    trade_state         varchar(48)     not null,
    bookkeeper          varchar(48)     not null,
    auction_house       varchar(48)     not null,
    wallet              varchar(48)     not null,
    token_account       varchar(48)     not null,
    metadata_mint       varchar(48)     not null,
    price               bigint          not null,
    token_size          bigint          not null,
    bump                smallint        not null,
    trade_state_bump    smallint        not null
);