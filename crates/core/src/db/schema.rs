table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    attributes (id) {
        metadata_address -> Varchar,
        value -> Nullable<Text>,
        trait_type -> Nullable<Text>,
        id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    auction_caches (address) {
        address -> Varchar,
        store_address -> Varchar,
        timestamp -> Timestamp,
        auction_data -> Varchar,
        auction_ext -> Varchar,
        vault -> Varchar,
        auction_manager -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    auction_datas (address) {
        address -> Varchar,
        ends_at -> Nullable<Timestamp>,
        authority -> Nullable<Varchar>,
        token_mint -> Nullable<Varchar>,
        store_owner -> Nullable<Varchar>,
        highest_bid -> Nullable<Int8>,
        end_auction_gap -> Nullable<Timestamp>,
        price_floor -> Nullable<Int8>,
        total_uncancelled_bids -> Nullable<Int4>,
        last_bid_time -> Nullable<Timestamp>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    auction_datas_ext (address) {
        address -> Varchar,
        gap_tick_size -> Nullable<Int4>,
        instant_sale_price -> Nullable<Int8>,
        name -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    bids (listing_address, bidder_address) {
        listing_address -> Varchar,
        bidder_address -> Varchar,
        last_bid_time -> Timestamp,
        last_bid_amount -> Int8,
        cancelled -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    editions (address) {
        address -> Varchar,
        parent_address -> Varchar,
        edition -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    files (id) {
        metadata_address -> Varchar,
        uri -> Text,
        file_type -> Text,
        id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    listing_metadatas (listing_address, metadata_address) {
        listing_address -> Varchar,
        metadata_address -> Varchar,
        metadata_index -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    master_editions (address) {
        address -> Varchar,
        supply -> Int8,
        max_supply -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    metadata_collections (id) {
        metadata_address -> Varchar,
        name -> Nullable<Text>,
        family -> Nullable<Text>,
        id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    metadata_creators (metadata_address, creator_address) {
        metadata_address -> Varchar,
        creator_address -> Varchar,
        share -> Int4,
        verified -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    metadata_jsons (metadata_address) {
        metadata_address -> Varchar,
        fingerprint -> Bytea,
        updated_at -> Timestamp,
        description -> Nullable<Text>,
        image -> Nullable<Text>,
        animation_url -> Nullable<Text>,
        external_url -> Nullable<Text>,
        category -> Nullable<Text>,
        raw_content -> Jsonb,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

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
        edition_pda -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    settings_uri_jsons (store_config_pda) {
        store_config_pda -> Varchar,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        logo_url -> Nullable<Text>,
        banner_url -> Nullable<Text>,
        subdomain -> Nullable<Text>,
        owner_address -> Nullable<Varchar>,
        auction_house_address -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    store_denylist (owner_address) {
        owner_address -> Varchar,
        hard_ban -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    storefronts (address) {
        owner_address -> Varchar,
        subdomain -> Text,
        title -> Text,
        description -> Text,
        favicon_url -> Text,
        logo_url -> Text,
        ts_index -> Tsvector,
        updated_at -> Nullable<Timestamp>,
        banner_url -> Nullable<Text>,
        address -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    storefrontsv2 (store_address) {
        store_address -> Varchar,
        public -> Nullable<Bool>,
        auction_program -> Nullable<Varchar>,
        token_vault_program -> Nullable<Varchar>,
        token_metadata_program -> Nullable<Varchar>,
        token_program -> Nullable<Varchar>,
        store_config_pda -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    storefrontsv2_configs (address) {
        address -> Varchar,
        settings_uri -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    token_accounts (address) {
        address -> Varchar,
        mint_address -> Varchar,
        owner_address -> Varchar,
        amount -> Nullable<Int8>,
        updated_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};

    whitelisted_creators (address, creator_address) {
        address -> Varchar,
        creator_address -> Varchar,
        activated -> Nullable<Bool>,
    }
}

allow_tables_to_appear_in_same_query!(
    attributes,
    auction_caches,
    auction_datas,
    auction_datas_ext,
    bids,
    editions,
    files,
    listing_metadatas,
    master_editions,
    metadata_collections,
    metadata_creators,
    metadata_jsons,
    metadatas,
    settings_uri_jsons,
    store_denylist,
    storefronts,
    storefrontsv2,
    storefrontsv2_configs,
    token_accounts,
    whitelisted_creators,
);
