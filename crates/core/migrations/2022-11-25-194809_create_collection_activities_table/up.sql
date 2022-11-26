create table if not exists collection_activities (
    id uuid primary key,
    metadata varchar(48) not null,
    price bigint not null,
    auction_house varchar(48) not null,
    created_at timestamp not null,
    marketplace_program varchar(48) not null,
    wallets text[] not null,
    collection_id text not null,
    activity_type text not null
);