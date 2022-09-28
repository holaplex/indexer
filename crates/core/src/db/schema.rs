table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    attributes (id) {
        metadata_address -> Varchar,
        value -> Nullable<Text>,
        trait_type -> Nullable<Text>,
        id -> Uuid,
        first_verified_creator -> Nullable<Varchar>,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    bonding_changes (address, slot) {
        address -> Varchar,
        insert_ts -> Timestamp,
        slot -> Int8,
        current_reserves_from_bonding -> Int8,
        current_supply_from_bonding -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    buy_instructions (id) {
        id -> Uuid,
        wallet -> Varchar,
        payment_account -> Varchar,
        transfer_authority -> Varchar,
        treasury_mint -> Varchar,
        token_account -> Varchar,
        metadata -> Varchar,
        escrow_payment_account -> Varchar,
        authority -> Varchar,
        auction_house -> Varchar,
        auction_house_fee_account -> Varchar,
        buyer_trade_state -> Varchar,
        trade_state_bump -> Int2,
        escrow_payment_bump -> Int2,
        buyer_price -> Int8,
        token_size -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cancel_instructions (id) {
        id -> Uuid,
        wallet -> Varchar,
        token_account -> Varchar,
        token_mint -> Varchar,
        authority -> Varchar,
        auction_house -> Varchar,
        auction_house_fee_account -> Varchar,
        trade_state -> Varchar,
        buyer_price -> Int8,
        token_size -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    candy_machine_collection_pdas (address) {
        address -> Varchar,
        mint -> Varchar,
        candy_machine -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    candy_machine_config_lines (candy_machine_address, idx) {
        candy_machine_address -> Varchar,
        name -> Text,
        uri -> Text,
        idx -> Int4,
        taken -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    candy_machine_creators (candy_machine_address, creator_address) {
        candy_machine_address -> Varchar,
        creator_address -> Varchar,
        verified -> Bool,
        share -> Int2,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    candy_machine_end_settings (candy_machine_address) {
        candy_machine_address -> Varchar,
        end_setting_type -> Settingtype,
        number -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    candy_machine_gate_keeper_configs (candy_machine_address) {
        candy_machine_address -> Varchar,
        gatekeeper_network -> Varchar,
        expire_on_use -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cardinal_claim_events (token_manager_address, state_changed_at) {
        token_manager_address -> Varchar,
        version -> Int2,
        bump -> Int2,
        count -> Int8,
        num_invalidators -> Int2,
        issuer -> Varchar,
        mint -> Varchar,
        amount -> Int8,
        kind -> Int2,
        state -> Int2,
        state_changed_at -> Timestamp,
        invalidation_type -> Int2,
        recipient_token_account -> Varchar,
        receipt_mint -> Nullable<Varchar>,
        claim_approver -> Nullable<Varchar>,
        transfer_authority -> Nullable<Varchar>,
        invalidators -> Nullable<Array<Text>>,
        paid_claim_approver_payment_amount -> Nullable<Int8>,
        paid_claim_approver_payment_mint -> Nullable<Varchar>,
        paid_claim_approver_payment_manager -> Nullable<Varchar>,
        paid_claim_approver_collector -> Nullable<Varchar>,
        time_invalidator_address -> Nullable<Varchar>,
        time_invalidator_payment_manager -> Nullable<Varchar>,
        time_invalidator_collector -> Nullable<Varchar>,
        time_invalidator_expiration -> Nullable<Timestamp>,
        time_invalidator_duration_seconds -> Nullable<Int8>,
        time_invalidator_extension_payment_amount -> Nullable<Int8>,
        time_invalidator_extension_duration_seconds -> Nullable<Int8>,
        time_invalidator_extension_payment_mint -> Nullable<Varchar>,
        time_invalidator_max_expiration -> Nullable<Timestamp>,
        time_invalidator_disable_partial_extension -> Nullable<Bool>,
        use_invalidator_address -> Nullable<Varchar>,
        use_invalidator_payment_manager -> Nullable<Varchar>,
        use_invalidator_collector -> Nullable<Varchar>,
        use_invalidator_usages -> Nullable<Int8>,
        use_invalidator_use_authority -> Nullable<Varchar>,
        use_invalidator_total_usages -> Nullable<Int8>,
        use_invalidator_extension_payment_amount -> Nullable<Int8>,
        use_invalidator_extension_payment_mint -> Nullable<Varchar>,
        use_invalidator_extension_usages -> Nullable<Int8>,
        use_invalidator_max_usages -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cardinal_entries (address) {
        address -> Varchar,
        namespace -> Varchar,
        name -> Text,
        data -> Nullable<Varchar>,
        reverse_entry -> Nullable<Varchar>,
        mint -> Varchar,
        is_claimed -> Bool,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cardinal_namespaces (address) {
        address -> Varchar,
        name -> Text,
        update_authority -> Varchar,
        rent_authority -> Varchar,
        approve_authority -> Nullable<Varchar>,
        schema -> Int2,
        payment_amount_daily -> Int8,
        payment_mint -> Varchar,
        min_rental_seconds -> Int8,
        max_rental_seconds -> Nullable<Int8>,
        transferable_entries -> Bool,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cardinal_paid_claim_approvers (paid_claim_approver_address) {
        paid_claim_approver_address -> Varchar,
        paid_claim_approver_bump -> Int2,
        paid_claim_approver_token_manager_address -> Varchar,
        paid_claim_approver_payment_manager -> Varchar,
        paid_claim_approver_payment_amount -> Int8,
        paid_claim_approver_payment_mint -> Varchar,
        paid_claim_approver_collector -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cardinal_time_invalidators (time_invalidator_address) {
        time_invalidator_address -> Varchar,
        time_invalidator_bump -> Int2,
        time_invalidator_token_manager_address -> Varchar,
        time_invalidator_payment_manager -> Nullable<Varchar>,
        time_invalidator_collector -> Nullable<Varchar>,
        time_invalidator_expiration -> Nullable<Timestamp>,
        time_invalidator_duration_seconds -> Nullable<Int8>,
        time_invalidator_extension_payment_amount -> Nullable<Int8>,
        time_invalidator_extension_duration_seconds -> Nullable<Int8>,
        time_invalidator_extension_payment_mint -> Nullable<Varchar>,
        time_invalidator_max_expiration -> Nullable<Timestamp>,
        time_invalidator_disable_partial_extension -> Nullable<Bool>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cardinal_token_manager_invalidators (token_manager_address, invalidator) {
        token_manager_address -> Varchar,
        invalidator -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
        state_changed_at -> Timestamp,
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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    cardinal_use_invalidators (use_invalidator_address) {
        use_invalidator_address -> Varchar,
        use_invalidator_bump -> Int2,
        use_invalidator_token_manager_address -> Varchar,
        use_invalidator_payment_manager -> Nullable<Varchar>,
        use_invalidator_collector -> Nullable<Varchar>,
        use_invalidator_usages -> Nullable<Int8>,
        use_invalidator_use_authority -> Nullable<Varchar>,
        use_invalidator_total_usages -> Nullable<Int8>,
        use_invalidator_extension_payment_amount -> Nullable<Int8>,
        use_invalidator_extension_payment_mint -> Nullable<Varchar>,
        use_invalidator_extension_usages -> Nullable<Int8>,
        use_invalidator_max_usages -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    collection_stats (collection_address) {
        collection_address -> Varchar,
        nft_count -> Int8,
        floor_price -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    collection_trends (collection) {
        collection -> Text,
        _1d_volume -> Nullable<Numeric>,
        _7d_volume -> Nullable<Numeric>,
        _30d_volume -> Nullable<Numeric>,
        _prev_1d_volume -> Nullable<Numeric>,
        _prev_7d_volume -> Nullable<Numeric>,
        _prev_30d_volume -> Nullable<Numeric>,
        _1d_sales_count -> Nullable<Numeric>,
        prev_1d_sales_count -> Nullable<Numeric>,
        _7d_sales_count -> Nullable<Numeric>,
        prev_7d_sales_count -> Nullable<Numeric>,
        _30d_sales_count -> Nullable<Numeric>,
        prev_30d_sales_count -> Nullable<Numeric>,
        floor_price -> Nullable<Numeric>,
        prev_1d_floor_price -> Nullable<Numeric>,
        prev_7d_floor_price -> Nullable<Numeric>,
        prev_30d_floor_price -> Nullable<Numeric>,
        _1d_volume_change -> Nullable<Int8>,
        _7d_volume_change -> Nullable<Int8>,
        _30d_volume_change -> Nullable<Int8>,
        _1d_floor_price_change -> Nullable<Int8>,
        _7d_floor_price_change -> Nullable<Int8>,
        _30d_floor_price_change -> Nullable<Int8>,
        _1d_sales_count_change -> Nullable<Int8>,
        _7d_sales_count_change -> Nullable<Int8>,
        _30d_sales_count_change -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    current_metadata_owners (mint_address) {
        mint_address -> Varchar,
        owner_address -> Varchar,
        token_account_address -> Varchar,
        updated_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    deposit_instructions (id) {
        id -> Uuid,
        wallet -> Varchar,
        payment_account -> Varchar,
        transfer_authority -> Varchar,
        escrow_payment_account -> Varchar,
        treasury_mint -> Varchar,
        authority -> Varchar,
        auction_house -> Varchar,
        auction_house_fee_account -> Varchar,
        escrow_payment_bump -> Int2,
        amount -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    editions (address) {
        address -> Varchar,
        parent_address -> Varchar,
        edition -> Int8,
        slot -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    execute_sale_instructions (id) {
        id -> Uuid,
        buyer -> Varchar,
        seller -> Varchar,
        token_account -> Varchar,
        token_mint -> Varchar,
        metadata -> Varchar,
        treasury_mint -> Varchar,
        escrow_payment_account -> Varchar,
        seller_payment_receipt_account -> Varchar,
        buyer_receipt_token_account -> Varchar,
        authority -> Varchar,
        auction_house -> Varchar,
        auction_house_fee_account -> Varchar,
        auction_house_treasury -> Varchar,
        buyer_trade_state -> Varchar,
        seller_trade_state -> Varchar,
        free_trade_state -> Varchar,
        program_as_signer -> Varchar,
        escrow_payment_bump -> Int2,
        free_trade_state_bump -> Int2,
        program_as_signer_bump -> Int2,
        buyer_price -> Int8,
        token_size -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    feed_event_wallets (wallet_address, feed_event_id) {
        wallet_address -> Varchar,
        feed_event_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    feed_events (id) {
        id -> Uuid,
        created_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    files (id) {
        metadata_address -> Varchar,
        uri -> Text,
        file_type -> Text,
        id -> Uuid,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    follow_events (feed_event_id) {
        graph_connection_address -> Varchar,
        feed_event_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    geno_habitat_datas (address) {
        address -> Varchar,
        habitat_mint -> Varchar,
        level -> Int2,
        element -> Int2,
        genesis -> Bool,
        renewal_timestamp -> Timestamp,
        expiry_timestamp -> Timestamp,
        next_day_timestamp -> Timestamp,
        crystals_refined -> Int2,
        harvester_bytes -> Bytea,
        ki_harvested -> Int8,
        seeds_spawned -> Bool,
        is_sub_habitat -> Bool,
        parent_habitat -> Nullable<Varchar>,
        sub_habitat_0 -> Nullable<Varchar>,
        sub_habitat_1 -> Nullable<Varchar>,
        harvester_royalty_bips -> Int4,
        harvester_open_market -> Bool,
        total_ki_harvested -> Int8,
        total_crystals_refined -> Int8,
        terraforming_habitat -> Nullable<Varchar>,
        active -> Bool,
        durability -> Int4,
        habitats_terraformed -> Int4,
        sequence -> Int8,
        guild -> Nullable<Int4>,
        sub_habitat_cooldown_timestamp -> Timestamp,
        harvester_settings_cooldown_timestamp -> Timestamp,
        slot -> Int8,
        write_version -> Int8,
        harvester -> Varchar,
        daily_ki_harvesting_cap -> Numeric,
        ki_available_to_harvest -> Nullable<Numeric>,
        has_max_ki -> Nullable<Bool>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    geno_rental_agreements (habitat_address) {
        habitat_address -> Varchar,
        alchemist -> Nullable<Varchar>,
        rental_period -> Int8,
        rent -> Int8,
        rent_token -> Varchar,
        rent_token_decimals -> Int2,
        last_rent_payment -> Timestamp,
        next_payment_due -> Timestamp,
        grace_period -> Int8,
        open_market -> Bool,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    governance_configs (governance_address) {
        governance_address -> Varchar,
        vote_threshold_type -> Votethresholdtype,
        vote_threshold_percentage -> Int2,
        min_community_weight_to_create_proposal -> Numeric,
        min_instruction_hold_up_time -> Int8,
        max_voting_time -> Int8,
        vote_tipping -> Votetipping,
        proposal_cool_off_time -> Int8,
        min_council_weight_to_create_proposal -> Int8,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    governances (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        realm -> Varchar,
        governed_account -> Varchar,
        proposals_count -> Int8,
        reserved -> Bytea,
        voting_proposal_count -> Int2,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    graph_connections (address) {
        address -> Varchar,
        from_account -> Varchar,
        to_account -> Varchar,
        connected_at -> Timestamp,
        disconnected_at -> Nullable<Timestamp>,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    ins_buffer_bundle_instructions (instruction_buffer_address, program_id) {
        instruction_buffer_address -> Varchar,
        program_id -> Varchar,
        data -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    ins_buffer_bundles (instruction_buffer_address) {
        instruction_buffer_address -> Varchar,
        is_executed -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    last_sold_metadatas (metadata) {
        metadata -> Varchar,
        purchase_id -> Nullable<Uuid>,
        price -> Nullable<Int8>,
        created_at -> Nullable<Timestamp>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    listing_denylist (listing_address) {
        listing_address -> Varchar,
        hard_ban -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    listing_events (feed_event_id) {
        feed_event_id -> Uuid,
        lifecycle -> Listingeventlifecycle,
        listing_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    listing_metadatas (listing_address, metadata_address) {
        listing_address -> Varchar,
        metadata_address -> Varchar,
        metadata_index -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    listings (id) {
        id -> Uuid,
        trade_state -> Varchar,
        auction_house -> Varchar,
        seller -> Varchar,
        metadata -> Varchar,
        purchase_id -> Nullable<Uuid>,
        price -> Int8,
        token_size -> Int8,
        trade_state_bump -> Int2,
        created_at -> Timestamp,
        canceled_at -> Nullable<Timestamp>,
        slot -> Int8,
        write_version -> Nullable<Int8>,
        marketplace_program -> Varchar,
        expiry -> Nullable<Timestamp>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    master_editions (address) {
        address -> Varchar,
        supply -> Int8,
        max_supply -> Nullable<Int8>,
        slot -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    me_collection_stats (collection_id) {
        collection_id -> Uuid,
        nft_count -> Int8,
        floor_price -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    me_collections (id) {
        symbol -> Nullable<Text>,
        name -> Nullable<Text>,
        family -> Nullable<Text>,
        id -> Uuid,
        image -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    me_metadata_collections (metadata_address, collection_id) {
        metadata_address -> Varchar,
        collection_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    metadata_collection_keys (metadata_address, collection_address) {
        metadata_address -> Varchar,
        collection_address -> Varchar,
        verified -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    metadata_collections (metadata_address) {
        metadata_address -> Varchar,
        name -> Nullable<Text>,
        family -> Nullable<Text>,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
        fetch_uri -> Text,
        slot -> Int8,
        write_version -> Int8,
        name -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
        slot -> Nullable<Int8>,
        burned_at -> Nullable<Timestamp>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    mint_events (feed_event_id) {
        metadata_address -> Varchar,
        feed_event_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    offer_events (feed_event_id) {
        feed_event_id -> Uuid,
        lifecycle -> Offereventlifecycle,
        offer_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    offers (id) {
        id -> Uuid,
        trade_state -> Varchar,
        auction_house -> Varchar,
        buyer -> Varchar,
        metadata -> Varchar,
        token_account -> Nullable<Varchar>,
        purchase_id -> Nullable<Uuid>,
        price -> Int8,
        token_size -> Int8,
        trade_state_bump -> Int2,
        created_at -> Timestamp,
        canceled_at -> Nullable<Timestamp>,
        slot -> Int8,
        write_version -> Nullable<Int8>,
        marketplace_program -> Varchar,
        expiry -> Nullable<Timestamp>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposal_instructions (proposal_address, program_id) {
        proposal_address -> Varchar,
        program_id -> Varchar,
        data -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposal_options (proposal_address, label) {
        proposal_address -> Varchar,
        label -> Text,
        vote_weight -> Int8,
        vote_result -> Optionvoteresult,
        transactions_executed_count -> Int2,
        transactions_count -> Int2,
        transactions_next_index -> Int2,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposal_transaction_instruction_accounts (proposal_transaction, account_pubkey) {
        proposal_transaction -> Varchar,
        account_pubkey -> Varchar,
        is_signer -> Bool,
        is_writable -> Bool,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposal_transaction_instructions (proposal_transaction, program_id, data) {
        proposal_transaction -> Varchar,
        program_id -> Varchar,
        data -> Bytea,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposal_transactions (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        proposal -> Varchar,
        option_index -> Int2,
        transaction_index -> Int2,
        hold_up_time -> Int8,
        executed_at -> Nullable<Timestamp>,
        execution_status -> Transactionexecutionstatus,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposal_vote_type_multi_choices (address) {
        address -> Varchar,
        max_voter_options -> Int2,
        max_winning_options -> Int2,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposals_v1 (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        governance -> Varchar,
        governing_token_mint -> Varchar,
        state -> Proposalstate,
        token_owner_record -> Varchar,
        signatories_count -> Int2,
        signatories_signed_off_count -> Int2,
        yes_votes_count -> Int8,
        no_votes_count -> Int8,
        instructions_executed_count -> Int2,
        instructions_count -> Int2,
        instructions_next_index -> Int2,
        draft_at -> Timestamp,
        signing_off_at -> Nullable<Timestamp>,
        voting_at -> Nullable<Timestamp>,
        voting_at_slot -> Nullable<Int8>,
        voting_completed_at -> Nullable<Timestamp>,
        executing_at -> Nullable<Timestamp>,
        closed_at -> Nullable<Timestamp>,
        execution_flags -> Instructionexecutionflags,
        max_vote_weight -> Nullable<Int8>,
        vote_threshold_type -> Nullable<Votethresholdtype>,
        vote_threshold_percentage -> Nullable<Int2>,
        name -> Text,
        description_link -> Text,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    proposals_v2 (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        governance -> Varchar,
        governing_token_mint -> Varchar,
        state -> Proposalstate,
        token_owner_record -> Varchar,
        signatories_count -> Int2,
        signatories_signed_off_count -> Int2,
        vote_type -> Proposalvotetype,
        deny_vote_weight -> Nullable<Int8>,
        veto_vote_weight -> Nullable<Int8>,
        abstain_vote_weight -> Nullable<Int8>,
        start_voting_at -> Nullable<Timestamp>,
        draft_at -> Timestamp,
        signing_off_at -> Nullable<Timestamp>,
        voting_at -> Nullable<Timestamp>,
        voting_at_slot -> Nullable<Int8>,
        voting_completed_at -> Nullable<Timestamp>,
        executing_at -> Nullable<Timestamp>,
        closed_at -> Nullable<Timestamp>,
        execution_flags -> Instructionexecutionflags,
        max_vote_weight -> Nullable<Int8>,
        max_voting_time -> Nullable<Int8>,
        vote_threshold_type -> Nullable<Votethresholdtype>,
        vote_threshold_percentage -> Nullable<Int2>,
        name -> Text,
        description_link -> Text,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    public_buy_instructions (id) {
        id -> Uuid,
        wallet -> Varchar,
        payment_account -> Varchar,
        transfer_authority -> Varchar,
        treasury_mint -> Varchar,
        token_account -> Varchar,
        metadata -> Varchar,
        escrow_payment_account -> Varchar,
        authority -> Varchar,
        auction_house -> Varchar,
        auction_house_fee_account -> Varchar,
        buyer_trade_state -> Varchar,
        trade_state_bump -> Int2,
        escrow_payment_bump -> Int2,
        buyer_price -> Int8,
        token_size -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    purchase_events (feed_event_id) {
        feed_event_id -> Uuid,
        purchase_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    purchases (id) {
        id -> Uuid,
        buyer -> Varchar,
        seller -> Varchar,
        auction_house -> Varchar,
        metadata -> Varchar,
        token_size -> Int8,
        price -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
        write_version -> Nullable<Int8>,
        marketplace_program -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    realm_config_accounts (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        realm -> Varchar,
        community_voter_weight_addin -> Nullable<Varchar>,
        max_community_voter_weight_addin -> Nullable<Varchar>,
        council_voter_weight_addin -> Nullable<Varchar>,
        council_max_vote_weight_addin -> Nullable<Varchar>,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    realm_configs (realm_address) {
        realm_address -> Varchar,
        use_community_voter_weight_addin -> Bool,
        use_max_community_voter_weight_addin -> Bool,
        reserved -> Bytea,
        min_community_weight_to_create_governance -> Numeric,
        community_mint_max_vote_weight_source -> Mintmaxvotetype,
        community_mint_max_vote_weight -> Int8,
        council_mint -> Nullable<Varchar>,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    realms (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        community_mint -> Varchar,
        reserved -> Bytea,
        voting_proposal_count -> Int2,
        authority -> Nullable<Varchar>,
        name -> Text,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    sell_instructions (id) {
        id -> Uuid,
        wallet -> Varchar,
        token_account -> Varchar,
        metadata -> Varchar,
        authority -> Varchar,
        auction_house -> Varchar,
        auction_house_fee_account -> Varchar,
        seller_trade_state -> Varchar,
        free_seller_trader_state -> Varchar,
        program_as_signer -> Varchar,
        trade_state_bump -> Int2,
        free_trade_state_bump -> Int2,
        program_as_signer_bump -> Int2,
        buyer_price -> Int8,
        token_size -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    signatory_records (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        proposal -> Varchar,
        signatory -> Varchar,
        signed_off -> Bool,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    smart_wallet_owners (smart_wallet_address, owner_address) {
        smart_wallet_address -> Varchar,
        owner_address -> Varchar,
        index -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    store_auction_houses (store_config_address, auction_house_address) {
        store_config_address -> Varchar,
        auction_house_address -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    store_config_jsons (config_address) {
        config_address -> Varchar,
        name -> Text,
        description -> Text,
        logo_url -> Text,
        banner_url -> Text,
        subdomain -> Text,
        owner_address -> Varchar,
        store_address -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    store_configs (address) {
        address -> Varchar,
        settings_uri -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    store_creators (store_config_address, creator_address) {
        store_config_address -> Varchar,
        creator_address -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    store_denylist (owner_address) {
        owner_address -> Varchar,
        hard_ban -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    stores (address) {
        address -> Varchar,
        public -> Bool,
        config_address -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    token_owner_records (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        realm -> Varchar,
        governing_token_mint -> Varchar,
        governing_token_owner -> Varchar,
        governing_token_deposit_amount -> Int8,
        unrelinquished_votes_count -> Int8,
        total_votes_count -> Int8,
        outstanding_proposal_count -> Int2,
        reserved -> Bytea,
        governance_delegate -> Nullable<Varchar>,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    twitter_handle_name_services (address) {
        address -> Varchar,
        wallet_address -> Varchar,
        twitter_handle -> Text,
        slot -> Int8,
        from_bonfida -> Bool,
        from_cardinal -> Bool,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    tx_instructions (transaction_address, program_id) {
        transaction_address -> Varchar,
        program_id -> Varchar,
        data -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    vote_record_v2_vote_approve_vote_choices (vote_record_v2_address, rank, weight_percentage) {
        vote_record_v2_address -> Varchar,
        rank -> Int2,
        weight_percentage -> Int2,
        slot -> Int8,
        write_version -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    vote_records_v1 (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        proposal -> Varchar,
        governing_token_owner -> Varchar,
        is_relinquished -> Bool,
        vote_type -> Voteweightv1,
        vote_weight -> Int8,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    vote_records_v2 (address) {
        address -> Varchar,
        account_type -> Governanceaccounttype,
        proposal -> Varchar,
        governing_token_owner -> Varchar,
        is_relinquished -> Bool,
        voter_weight -> Int8,
        vote -> Vote_record_v2_vote,
        slot -> Int8,
        write_version -> Int8,
        program_id -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

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
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    wallet_totals (address) {
        address -> Varchar,
        following -> Int8,
        followers -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    whitelisted_creators (address) {
        address -> Varchar,
        creator_address -> Varchar,
        activated -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    withdraw_from_fee_instructions (id) {
        id -> Uuid,
        authority -> Varchar,
        fee_withdrawal_destination -> Varchar,
        auction_house_fee_account -> Varchar,
        auction_house -> Varchar,
        amount -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    withdraw_from_treasury_instructions (id) {
        id -> Uuid,
        treasury_mint -> Varchar,
        authority -> Varchar,
        treasury_withdrawal_destination -> Varchar,
        auction_house_treasury -> Varchar,
        auction_house -> Varchar,
        amount -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::{TsVector as Tsvector, TsQuery as Tsquery};
    use crate::db::custom_types::{ListingEventLifecycle as Listingeventlifecycle, Mode, ProposalState as Proposalstate, InstructionExecutionFlags as Instructionexecutionflags, ProposalVoteType as Proposalvotetype, OptionVoteResult as Optionvoteresult, MintMaxVoteType as Mintmaxvotetype, VoteTipping as Votetipping, VoteWeightV1 as Voteweightv1, VoteRecordV2Vote as Vote_record_v2_vote, VoteThresholdType as Votethresholdtype, GovernanceAccountType as Governanceaccounttype, TransactionExecutionStatus as Transactionexecutionstatus, OfferEventLifecycle as Offereventlifecycle, SettingType as Settingtype, TokenStandard as Token_standard, };

    withdraw_instructions (id) {
        id -> Uuid,
        wallet -> Varchar,
        receipt_account -> Varchar,
        escrow_payment_account -> Varchar,
        treasury_mint -> Varchar,
        authority -> Varchar,
        auction_house -> Varchar,
        auction_house_fee_account -> Varchar,
        escrow_payment_bump -> Int2,
        amount -> Int8,
        created_at -> Timestamp,
        slot -> Int8,
    }
}

joinable!(cardinal_token_manager_invalidators -> cardinal_token_managers (token_manager_address));
joinable!(feed_event_wallets -> feed_events (feed_event_id));
joinable!(follow_events -> feed_events (feed_event_id));
joinable!(follow_events -> graph_connections (graph_connection_address));
joinable!(governance_configs -> governances (governance_address));
joinable!(listing_events -> feed_events (feed_event_id));
joinable!(mint_events -> feed_events (feed_event_id));
joinable!(offer_events -> feed_events (feed_event_id));
joinable!(purchase_events -> feed_events (feed_event_id));
joinable!(realm_configs -> realms (realm_address));

allow_tables_to_appear_in_same_query!(
    attributes,
    auction_caches,
    auction_datas,
    auction_datas_ext,
    auction_houses,
    bid_receipts,
    bids,
    bonding_changes,
    buy_instructions,
    cancel_instructions,
    candy_machine_collection_pdas,
    candy_machine_config_lines,
    candy_machine_creators,
    candy_machine_datas,
    candy_machine_end_settings,
    candy_machine_gate_keeper_configs,
    candy_machine_hidden_settings,
    candy_machine_whitelist_mint_settings,
    candy_machines,
    cardinal_claim_events,
    cardinal_entries,
    cardinal_namespaces,
    cardinal_paid_claim_approvers,
    cardinal_time_invalidators,
    cardinal_token_manager_invalidators,
    cardinal_token_managers,
    cardinal_use_invalidators,
    collection_stats,
    collection_trends,
    current_metadata_owners,
    deposit_instructions,
    editions,
    escrows,
    execute_sale_instructions,
    feed_event_wallets,
    feed_events,
    files,
    follow_events,
    geno_habitat_datas,
    geno_rental_agreements,
    governance_configs,
    governance_parameters,
    governances,
    governors,
    graph_connections,
    ins_buffer_bundle_ins_keys,
    ins_buffer_bundle_instructions,
    ins_buffer_bundles,
    instruction_buffers,
    last_sold_metadatas,
    listing_denylist,
    listing_events,
    listing_metadatas,
    listing_receipts,
    listings,
    locker_params,
    locker_whitelist_entries,
    lockers,
    master_editions,
    me_collection_stats,
    me_collections,
    me_metadata_collections,
    metadata_collection_keys,
    metadata_collections,
    metadata_creators,
    metadata_jsons,
    metadatas,
    mint_events,
    offer_events,
    offers,
    proposal_account_metas,
    proposal_instructions,
    proposal_metas,
    proposal_options,
    proposal_transaction_instruction_accounts,
    proposal_transaction_instructions,
    proposal_transactions,
    proposal_vote_type_multi_choices,
    proposals,
    proposals_v1,
    proposals_v2,
    public_buy_instructions,
    purchase_events,
    purchase_receipts,
    purchases,
    realm_config_accounts,
    realm_configs,
    realms,
    sell_instructions,
    signatory_records,
    smart_wallet_owners,
    smart_wallets,
    store_auction_houses,
    store_config_jsons,
    store_configs,
    store_creators,
    store_denylist,
    storefronts,
    stores,
    sub_account_infos,
    token_owner_records,
    transactions,
    twitter_handle_name_services,
    tx_instruction_keys,
    tx_instructions,
    vote_record_v2_vote_approve_vote_choices,
    vote_records_v1,
    vote_records_v2,
    votes,
    wallet_totals,
    whitelisted_creators,
    withdraw_from_fee_instructions,
    withdraw_from_treasury_instructions,
    withdraw_instructions,
);
