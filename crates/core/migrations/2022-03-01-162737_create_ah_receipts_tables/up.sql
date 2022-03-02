create table bid_receipts (
    address                 varchar(48) primary key,
    trade_state             varchar(48) not null,
    bookkeeper              varchar(48) not null,
    auction_house           varchar(48) not null,
    buyer                   varchar(48) not null,
    metadata                varchar(48) not null,
    token_account           varchar(48),
    purchase_receipt        varchar(48),
    price                   bigint      not null,
    token_size              bigint      not null,
    bump                    smallint    not null,
    trade_state_bump        smallint    not null,
    created_at              timestamp   not null,
    canceled_at             timestamp
);

create table listing_receipts (
    address             varchar(48)     primary key,
    trade_state         varchar(48)     not null,
    bookkeeper          varchar(48)     not null,
    auction_house       varchar(48)     not null,
    seller              varchar(48)     not null,
    metadata            varchar(48)     not null,
    purchase_receipt    varchar(48),
    price               bigint          not null,
    token_size          bigint          not null,
    bump                smallint        not null,
    trade_state_bump    smallint        not null,
    created_at          timestamp       not null,
    canceled_at         timestamp    
);

create table purchase_receipts(
    address         varchar(48)     primary key,
    bookkeeper      varchar(48)     not null,
    buyer           varchar(48)     not null,
    seller          varchar(48)     not null,
    auction_house   varchar(48)     not null,
    metadata        varchar(48)     not null,
    token_size      bigint          not null,
    price           bigint          not null,
    bump            smallint        not null,
    created_at      timestamp       not null
);

create index if not exists bid_receipts_address_index
on bid_receipts (address);

create index if not exists bid_receipts_auction_house_index
on bid_receipts (auction_house);

create index if not exists bid_receipts_metadata_index
on bid_receipts (metadata);

create index if not exists listing_receipts_address_index
on listing_receipts (address);

create index if not exists listing_receipts_auction_house_index
on listing_receipts (auction_house);

create index if not exists listing_receipts_metadata_index
on listing_receipts (metadata);

create index if not exists purchase_receipts_address_index
on purchase_receipts (address);

create index if not exists purchase_receipts_auction_house_index
on purchase_receipts (auction_house);

create index if not exists purchase_receipts_metadata_index
on purchase_receipts (metadata);
