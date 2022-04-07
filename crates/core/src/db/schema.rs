table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    attributes (id) {
        metadata_address -> Varchar,
        value -> Nullable<Text>,
        trait_type -> Nullable<Text>,
        id -> Uuid,
        first_verified_creator -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    auction_houses (address) {
        address -> Varchar,
        treasury_mint -> Varchar,
        auction_house_treasury -> Varchar,
        treasury_withdrawal_destination -> Varchar,
        fee_withdrawal_destination -> Varchar,
        authority -> Varchar,
        creator -> Varchar,
        bump -> Int2,
        treasury_bump -> Int2,
        fee_payer_bump -> Int2,
        seller_fee_basis_points -> Int2,
        requires_sign_off -> Bool,
        can_change_sale_price -> Bool,
        auction_house_fee_account -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    bid_receipts (address) {
        address -> Varchar,
        trade_state -> Varchar,
        bookkeeper -> Varchar,
        auction_house -> Varchar,
        buyer -> Varchar,
        metadata -> Varchar,
        token_account -> Nullable<Varchar>,
        purchase_receipt -> Nullable<Varchar>,
        price -> Int8,
        token_size -> Int8,
        bump -> Int2,
        trade_state_bump -> Int2,
        created_at -> Timestamp,
        canceled_at -> Nullable<Timestamp>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_collection_pdas (address) {
        address -> Varchar,
        mint -> Varchar,
        candy_machine -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_config_lines (address) {
        address -> Varchar,
        name -> Text,
        uri -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_creators (candy_machine_address) {
        candy_machine_address -> Varchar,
        creator_address -> Varchar,
        verified -> Bool,
        share -> Int2,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_datas (candy_machine_address) {
        candy_machine_address -> Varchar,
        uuid -> Text,
        price -> Int8,
        symbol -> Text,
        seller_fee_basis_points -> Int2,
        max_supply -> Int8,
        is_mutable -> Bool,
        retain_authority -> Bool,
        go_live_date -> Nullable<Int8>,
        items_available -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_end_settings (candy_machine_address) {
        candy_machine_address -> Varchar,
        end_setting_type -> Settingtype,
        number -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_gate_keeper_configs (candy_machine_address) {
        candy_machine_address -> Varchar,
        gatekeeper_network -> Varchar,
        expire_on_use -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_hidden_settings (candy_machine_address) {
        candy_machine_address -> Varchar,
        name -> Text,
        uri -> Text,
        hash -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machine_whitelist_mint_settings (candy_machine_address) {
        candy_machine_address -> Varchar,
        mode -> Mode,
        mint -> Varchar,
        presale -> Bool,
        discount_price -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    candy_machines (address) {
        address -> Varchar,
        authority -> Varchar,
        wallet -> Varchar,
        token_mint -> Nullable<Varchar>,
        items_redeemed -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    editions (address) {
        address -> Varchar,
        parent_address -> Varchar,
        edition -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    graph_connections (address) {
        address -> Varchar,
        from_account -> Varchar,
        to_account -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    listing_denylist (listing_address) {
        listing_address -> Varchar,
        hard_ban -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    listing_metadatas (listing_address, metadata_address) {
        listing_address -> Varchar,
        metadata_address -> Varchar,
        metadata_index -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    listing_receipts (address) {
        address -> Varchar,
        trade_state -> Varchar,
        bookkeeper -> Varchar,
        auction_house -> Varchar,
        seller -> Varchar,
        metadata -> Varchar,
        purchase_receipt -> Nullable<Varchar>,
        price -> Int8,
        token_size -> Int8,
        bump -> Int2,
        trade_state_bump -> Int2,
        created_at -> Timestamp,
        canceled_at -> Nullable<Timestamp>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    master_editions (address) {
        address -> Varchar,
        supply -> Int8,
        max_supply -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    metadata_collection_keys (metadata_address, collection_address) {
        metadata_address -> Varchar,
        collection_address -> Varchar,
        verified -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    metadata_creators (metadata_address, creator_address) {
        metadata_address -> Varchar,
        creator_address -> Varchar,
        share -> Int4,
        verified -> Bool,
        position -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
        model -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
        token_standard -> Nullable<Token_standard>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    purchase_receipts (address) {
        address -> Varchar,
        bookkeeper -> Varchar,
        buyer -> Varchar,
        seller -> Varchar,
        auction_house -> Varchar,
        metadata -> Varchar,
        token_size -> Int8,
        price -> Int8,
        bump -> Int2,
        created_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    store_config_jsons (config_address) {
        config_address -> Varchar,
        name -> Text,
        description -> Text,
        logo_url -> Text,
        banner_url -> Text,
        subdomain -> Text,
        owner_address -> Varchar,
        auction_house_address -> Varchar,
        store_address -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    store_configs (address) {
        address -> Varchar,
        settings_uri -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    store_creators (store_config_address, creator_address) {
        store_config_address -> Varchar,
        creator_address -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    store_denylist (owner_address) {
        owner_address -> Varchar,
        hard_ban -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

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
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    stores (address) {
        address -> Varchar,
        public -> Bool,
        config_address -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    token_accounts (address) {
        address -> Varchar,
        mint_address -> Varchar,
        owner_address -> Varchar,
        amount -> Int8,
        updated_at -> Timestamp,
        slot -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    twitter_handle_name_services (address) {
        address -> Varchar,
        wallet_address -> Varchar,
        twitter_handle -> Text,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    whitelisted_creators (address) {
        address -> Varchar,
        creator_address -> Varchar,
        activated -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode};

    cardinal_token_managers (address) {
        address -> Varchar,
        version -> Int2,
        bump -> Int2,
        count -> Int8,
        num_invalidators -> Int2,
        issuer -> Varchar,
        mint -> Varchar,
        amount -> Int8,
        kind -> Int2,
        state -> Int2,
        state_changed_at -> Nullable<Timestamp>,
        invalidation_type -> Int2,
        recipient_token_account -> Varchar,
        receipt_mint -> Nullable<Varchar>,
        claim_approver -> Nullable<Varchar>,
        transfer_authority -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode};

    cardinal_paid_claim_approvers (address) {
        program -> Varchar,
        address -> Varchar,
        bump -> Int2,
        token_manager_address -> Varchar,
        payment_amount -> Int8,
        payment_mint -> Varchar,
        collector -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode};

    cardinal_token_manager_invalidators (token_manager_address, invalidator) {
        token_manager_address -> Varchar,
        invalidator -> Varchar,
        program -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode};

    cardinal_time_invalidators (address) {
        program -> Varchar,
        address -> Varchar,
        bump -> Int2,
        token_manager_address -> Varchar,
        expiration -> Nullable<Timestamp>,
        duration_seconds -> Nullable<Int8>,
        extension_payment_amount -> Nullable<Int8>,
        extension_duration_seconds -> Nullable<Int8>,
        extension_payment_mint -> Nullable<Varchar>,
        max_expiration -> Nullable<Timestamp>,
        disable_partial_extension -> Nullable<Bool>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode};

    cardinal_use_invalidators (address) {
        program -> Varchar,
        address -> Varchar,
        bump -> Int2,
        token_manager_address -> Varchar,
        use_authority -> Nullable<Varchar>,
        usages -> Int8,
        total_usages -> Int8,
        extension_payment_amount -> Int8,
        extension_payment_mint -> Varchar,
        extension_usages -> Int8,
        max_usages -> Nullable<Int8>,
    }
}

allow_tables_to_appear_in_same_query!(
    attributes,
    auction_caches,
    auction_datas,
    auction_datas_ext,
    auction_houses,
    bid_receipts,
    bids,
    candy_machine_collection_pdas,
    candy_machine_config_lines,
    candy_machine_creators,
    candy_machine_datas,
    candy_machine_end_settings,
    candy_machine_gate_keeper_configs,
    candy_machine_hidden_settings,
    candy_machine_whitelist_mint_settings,
    candy_machines,
    editions,
    files,
    graph_connections,
    listing_denylist,
    listing_metadatas,
    listing_receipts,
    master_editions,
    metadata_collection_keys,
    metadata_collections,
    metadata_creators,
    metadata_jsons,
    metadatas,
    purchase_receipts,
    store_config_jsons,
    store_configs,
    store_creators,
    store_denylist,
    storefronts,
    stores,
    token_accounts,
    twitter_handle_name_services,
    whitelisted_creators,
    cardinal_token_managers,
    cardinal_time_invalidators,
    cardinal_use_invalidators,
    cardinal_paid_claim_approvers,
    cardinal_token_manager_invalidators,
);
