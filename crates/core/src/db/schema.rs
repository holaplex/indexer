table! {
    editions (address) {
        address -> Varchar,
        parent_address -> Varchar,
        edition -> Int8,
    }
}

table! {
    listing_metadatas (listing_address, metadata_address) {
        listing_address -> Varchar,
        metadata_address -> Varchar,
        metadata_index -> Int4,
    }
}

table! {
    listings (address) {
        address -> Varchar,
        ends_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        ended -> Bool,
        authority -> Varchar,
        token_mint -> Varchar,
        store_owner -> Varchar,
        last_bid -> Nullable<Int8>,
        end_auction_gap -> Nullable<Timestamp>,
        price_floor -> Nullable<Int8>,
        total_uncancelled_bids -> Nullable<Int4>,
        gap_tick_size -> Nullable<Int4>,
        instant_sale_price -> Nullable<Int8>,
        name -> Text,
        last_bid_time -> Nullable<Timestamp>,
    }
}

table! {
    master_editions (address) {
        address -> Varchar,
        supply -> Int8,
        max_supply -> Nullable<Int8>,
    }
}

table! {
    metadata_creators (metadata_address, creator_address) {
        metadata_address -> Varchar,
        creator_address -> Varchar,
        share -> Int4,
        verified -> Bool,
    }
}

table! {
    metadatas (address) {
        address -> Varchar,
        name -> Text,
        symbol -> Text,
        uri -> Text,
        seller_fee_basis_points -> Int4,
        update_authority_address -> Varchar,
        mint_address -> Varchar,
        primary_sale_happened -> Bool,
        is_mutable -> Bool,
        edition_nonce -> Nullable<Int4>,
    }
}

table! {
    storefronts (owner_address) {
        owner_address -> Varchar,
        subdomain -> Text,
        title -> Text,
        description -> Text,
        favicon_url -> Text,
        logo_url -> Text,
    }
}

joinable!(editions -> master_editions (parent_address));
joinable!(listing_metadatas -> listings (listing_address));
joinable!(listing_metadatas -> metadatas (metadata_address));
joinable!(listings -> storefronts (store_owner));
joinable!(metadata_creators -> metadatas (metadata_address));

allow_tables_to_appear_in_same_query!(
    editions,
    listing_metadatas,
    listings,
    master_editions,
    metadata_creators,
    metadatas,
    storefronts,
);
