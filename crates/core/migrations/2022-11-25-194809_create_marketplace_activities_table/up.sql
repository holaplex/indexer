DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'activity_type') THEN
       create type activity_type as enum 
       ('ListingCreated', 'ListingCanceled', 'OfferCreated', 'OfferCanceled', 'Purchase');
    END IF;
END$$;

create table if not exists marketplace_activities (
    id uuid default gen_random_uuid() primary key,
    activity_id uuid not null,
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

create index if not exists marketplace_activities_metadata_idx on marketplace_activities(metadata);
create index if not exists marketplace_activities_created_at_idx on marketplace_activities(created_at);
create index if not exists marketplace_activities_buyer_idx on marketplace_activities(buyer);
create index if not exists marketplace_activities_seller_idx on marketplace_activities(seller);
create index if not exists marketplace_activities_collection_id_idx on marketplace_activities(collection_id);
create index if not exists marketplace_activities_activity_type_idx on marketplace_activities(activity_type);