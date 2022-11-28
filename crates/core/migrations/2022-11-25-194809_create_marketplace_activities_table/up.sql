create type activity_type 
as enum ('ListingCreated', 'ListingCanceled', 'OfferCreated', 'OfferCanceled', 'Purchase');

create table if not exists marketplace_activities (
    id uuid primary key,
    metadata varchar(48) not null,
    price bigint not null,
    auction_house varchar(48) not null,
    created_at timestamp not null,
    marketplace_program varchar(48) not null,
    buyer varchar(48),
    seller varchar(48),
    collection_id text,
    activity_type activity_type not null
);