create table public_bids (
    address                 varchar(48) primary key,
    trade_state             varchar(48) not null,
    bookkeeper              varchar(48) not null,
    auction_house           varchar(48) not null,
    wallet                  varchar(48) not null,
    token_mint              varchar(48) not null,
    price                   bigint      not null,
    token_size              bigint      not null,
    bump                    smallint    not null,
    trade_state_bump        smallint    not null,
    activated_at            bigint,
    closed_at               bigint
);

  
create table listings (
    address             varchar(48)     primary key,
    trade_state         varchar(48)     not null,
    bookkeeper          varchar(48)     not null,
    auction_house       varchar(48)     not null,
    seller              varchar(48)     not null,
    token_mint          varchar(48)     not null,
    price               bigint          not null,
    token_size          bigint          not null,
    bump                smallint        not null,
    trade_state_bump    smallint        not null,
    activated_at           bigint,
    closed_at           bigint    
);

create table purchases(
    address         varchar(48)     primary key,
    buyer           varchar(48)     not null,
    seller          varchar(48)     not null,
    auction_house   varchar(48)     not null,
    token_mint      varchar(48)     not null,
    token_size      bigint          not null,
    price           bigint          not null,
    bump            smallint        not null,
    created_at      bigint
);


create index if not exists public_bids_address_index
on public_bids (address);

create index if not exists public_bids_auction_house_index
on public_bids (auction_house);

create index if not exists listings_address_index
on listings (address);

create index if not exists listings_auction_house_index
on listings (auction_house);

create index if not exists purchases_address_index
on purchases (address);

create index if not exists purchases_auction_house_index
on listings (auction_house);