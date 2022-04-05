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

    escrows (address) {
        address -> Varchar,
        locker -> Varchar,
        owner -> Varchar,
        bump -> Int2,
        tokens -> Varchar,
        amount -> Int8,
        escrow_started_at -> Int8,
        escrow_ends_at -> Int8,
        vote_delegate -> Varchar,
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

    governance_parameters (governor_address) {
        governor_address -> Varchar,
        voting_delay -> Int8,
        voting_period -> Int8,
        quorum_votes -> Int8,
        timelock_delay_seconds -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    governors (address) {
        address -> Varchar,
        base -> Varchar,
        bump -> Int2,
        proposal_count -> Int8,
        electorate -> Varchar,
        smart_wallet -> Varchar,
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

    ins_buffer_bundle_ins_keys (instruction_buffer_address, program_id, pubkey) {
        instruction_buffer_address -> Varchar,
        program_id -> Varchar,
        pubkey -> Varchar,
        is_signer -> Bool,
        is_writable -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    ins_buffer_bundle_instructions (instruction_buffer_address, program_id) {
        instruction_buffer_address -> Varchar,
        program_id -> Varchar,
        data -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    ins_buffer_bundles (instruction_buffer_address) {
        instruction_buffer_address -> Varchar,
        is_executed -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    instruction_buffers (address) {
        address -> Varchar,
        owner_set_seqno -> Int8,
        eta -> Int8,
        authority -> Varchar,
        executor -> Varchar,
        smart_wallet -> Varchar,
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

    locker_params (locker_address) {
        locker_address -> Varchar,
        whitelist_enabled -> Bool,
        max_stake_vote_multiplier -> Int2,
        min_stake_duration -> Int8,
        max_stake_duration -> Int8,
        proposal_activation_min_votes -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    locker_whitelist_entries (address) {
        address -> Varchar,
        bump -> Int2,
        locker -> Varchar,
        program_id -> Varchar,
        owner -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    lockers (address) {
        address -> Varchar,
        base -> Varchar,
        bump -> Int2,
        token_mint -> Varchar,
        locked_supply -> Int8,
        governor -> Varchar,
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

    proposal_account_metas (proposal_address, program_id, pubkey) {
        proposal_address -> Varchar,
        program_id -> Varchar,
        pubkey -> Varchar,
        is_signer -> Bool,
        is_writable -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    proposal_instructions (proposal_address, program_id) {
        proposal_address -> Varchar,
        program_id -> Varchar,
        data -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    proposal_metas (address) {
        address -> Varchar,
        proposal -> Varchar,
        title -> Text,
        description_link -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    proposals (address) {
        address -> Varchar,
        governor -> Varchar,
        index -> Int8,
        bump -> Int2,
        proposer -> Varchar,
        quorum_votes -> Int8,
        for_votes -> Int8,
        against_votes -> Int8,
        abstain_votes -> Int8,
        canceled_at -> Int8,
        created_at -> Int8,
        activated_at -> Int8,
        voting_ends_at -> Int8,
        queued_at -> Int8,
        queued_transaction -> Varchar,
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

    smart_wallet_owners (smart_wallet_address, owner_address) {
        smart_wallet_address -> Varchar,
        owner_address -> Varchar,
        index -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    smart_wallets (address) {
        address -> Varchar,
        base -> Varchar,
        bump -> Int2,
        threshold -> Int8,
        minimum_delay -> Int8,
        grace_period -> Int8,
        owner_set_seqno -> Int8,
        num_transactions -> Int8,
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

    sub_account_infos (address) {
        address -> Varchar,
        smart_wallet -> Varchar,
        subaccount_type -> Int2,
        index -> Int8,
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

    transactions (address) {
        address -> Varchar,
        smart_wallet -> Varchar,
        index -> Int8,
        bump -> Int2,
        proposer -> Varchar,
        signers -> Array<Bool>,
        owner_set_seqno -> Int8,
        eta -> Int8,
        executor -> Varchar,
        executed_at -> Int8,
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

    tx_instruction_keys (transaction_address, program_id, pubkey) {
        transaction_address -> Varchar,
        program_id -> Varchar,
        pubkey -> Varchar,
        is_signer -> Bool,
        is_writable -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    tx_instructions (transaction_address, program_id) {
        transaction_address -> Varchar,
        program_id -> Varchar,
        data -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{SettingType as Settingtype, Mode, TokenStandard as Token_standard};

    votes (address) {
        address -> Varchar,
        proposal -> Varchar,
        voter -> Varchar,
        bump -> Int2,
        side -> Int2,
        weight -> Int8,
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
    escrows,
    files,
    governance_parameters,
    governors,
    graph_connections,
    ins_buffer_bundle_ins_keys,
    ins_buffer_bundle_instructions,
    ins_buffer_bundles,
    instruction_buffers,
    listing_denylist,
    listing_metadatas,
    listing_receipts,
    locker_params,
    locker_whitelist_entries,
    lockers,
    master_editions,
    metadata_collection_keys,
    metadata_collections,
    metadata_creators,
    metadata_jsons,
    metadatas,
    proposal_account_metas,
    proposal_instructions,
    proposal_metas,
    proposals,
    purchase_receipts,
    smart_wallet_owners,
    smart_wallets,
    store_config_jsons,
    store_configs,
    store_creators,
    store_denylist,
    storefronts,
    stores,
    sub_account_infos,
    token_accounts,
    transactions,
    twitter_handle_name_services,
    tx_instruction_keys,
    tx_instructions,
    votes,
    whitelisted_creators,
);
