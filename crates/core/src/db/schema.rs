table! {
    editions (address) {
        address -> Bytea,
        parent_address -> Bytea,
        edition -> Int8,
    }
}

table! {
    listing_metadatas (listing_address, metadata_address) {
        listing_address -> Bytea,
        metadata_address -> Bytea,
    }
}

table! {
    listings (address) {
        address -> Bytea,
        ends_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        ended -> Bool,
        authority -> Bytea,
        token_mint -> Bytea,
        store_owner -> Bytea,
        last_bid -> Nullable<Int8>,
        end_auction_gap -> Nullable<Timestamp>,
        price_floor -> Nullable<Int8>,
        total_uncancelled_bids -> Nullable<Int4>,
        gap_tick_size -> Nullable<Int4>,
        instant_sale_price -> Nullable<Int8>,
        name -> Text,
    }
}

table! {
    master_editions (address) {
        address -> Bytea,
        supply -> Int8,
        max_supply -> Nullable<Int8>,
    }
}

table! {
    metadata_creators (metadata_address, creator_address) {
        metadata_address -> Bytea,
        creator_address -> Bytea,
        share -> Int4,
        verified -> Bool,
    }
}

table! {
    metadatas (address) {
        address -> Bytea,
        name -> Text,
        symbol -> Text,
        uri -> Text,
        seller_fee_basis_points -> Int4,
        update_authority_address -> Bytea,
        mint_address -> Bytea,
        primary_sale_happened -> Bool,
        is_mutable -> Bool,
        edition_nonce -> Nullable<Int4>,
    }
}

joinable!(editions -> master_editions (parent_address));
joinable!(listing_metadatas -> listings (listing_address));
joinable!(listing_metadatas -> metadatas (metadata_address));
joinable!(metadata_creators -> metadatas (metadata_address));

allow_tables_to_appear_in_same_query!(
    editions,
    listing_metadatas,
    listings,
    master_editions,
    metadata_creators,
    metadatas,
);
