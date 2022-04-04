-- Tribeca locked_voter program accounts tables
create table lockers (
    address                         varchar(48)             primary key,
    base                            varchar(48)             not null,
    bump                            smallint                not null,
    token_mint                      varchar(48)             not null,
    locked_supply                   bigint                  not null,
    governor                        varchar(48)             not null
);

create index if not exists locker_base_idx on 
lockers using hash (base);

create index if not exists locker_governor_idx on 
lockers using hash (governor);

create table locker_params (
    locker_address                  varchar(48)             primary key,
    whitelist_enabled               bool                    not null,
    max_stake_vote_multiplier       smallint                not null,
    min_stake_duration              bigint                  not null,
    max_stake_duration              bigint                  not null,
    proposal_activation_min_votes   bigint                  not null
);

create table locker_whitelist_entries (
    address                         varchar(48)             primary key,
    bump                            smallint                not null,
    locker                          varchar(48)             not null,
    program_id                      varchar(48)             not null,
    owner                           varchar(48)             not null
);

create index if not exists locker_whitelist_entries_locker_idx on 
locker_whitelist_entries using hash (locker);

create index if not exists locker_whitelist_entries_program_id_idx on 
locker_whitelist_entries using hash (program_id);

create index if not exists locker_whitelist_entries_owner_idx on 
locker_whitelist_entries using hash (owner);

create table escrows (
    address                         varchar(48)             primary key,
    locker                          varchar(48)             not null,
    owner                           varchar(48)             not null,
    bump                            smallint                not null,
    tokens                          varchar(48)             not null,
    amount                          bigint                  not null,
    escrow_started_at               bigint                  not null,
    escrow_ends_at                  bigint                  not null,
    vote_delegate                   varchar(48)             not null
);

create index if not exists escrows_locker_idx on 
escrows using hash (locker);

create index if not exists escrows_owner_idx on 
escrows using hash (owner);

create index if not exists escrows_vote_delegate_idx on 
escrows using hash (vote_delegate);

-- Tribeca govern program accounts tables
create table governors (
    address                         varchar(48)             primary key,
    base                            varchar(48)             not null,
    bump                            smallint                not null,
    proposal_count                  bigint                  not null,
    electorate                      varchar(48)             not null,
    smart_wallet                    varchar(48)             not null
);

create index if not exists governors_base_idx on 
governors using hash (base);

create index if not exists governors_electorate_idx on 
governors using hash (smart_wallet);

create index if not exists governors_smart_wallet_idx on 
governors using hash (electorate);

create table governance_parameters (
    governor_address                varchar(48)             primary key,
    voting_delay                    bigint                  not null,
    voting_period                   bigint                  not null,
    quorum_votes                    bigint                  not null,
    timelock_delay_seconds          bigint                  not null
);

create table proposals (
    address                         varchar(48)             primary key,
    governor                        varchar(48)             not null,
    index                           bigint                  not null,
    bump                            smallint                not null,
    proposer                        varchar(48)             not null,
    quorum_votes                    bigint                  not null,
    for_votes                       bigint                  not null,
    against_votes                   bigint                  not null,
    abstain_votes                   bigint                  not null,
    canceled_at                     bigint                  not null,
    created_at                      bigint                  not null,
    activated_at                    bigint                  not null,
    voting_ends_at                  bigint                  not null,
    queued_at                       bigint                  not null,
    queued_transaction              varchar(48)             not null
);

create index if not exists proposals_governor_idx on 
proposals using hash (governor);

create index if not exists proposals_proposer_idx on 
proposals using hash (proposer);

create index if not exists proposals_queued_transaction_idx on 
proposals using hash (queued_transaction);

create table proposal_instructions (
    proposal_address                varchar(48)             primary key,
    program_id                      varchar(48)             not null,
    data                            bytea                   not null
);

create index if not exists proposal_instructions_program_id_idx on 
proposal_instructions using hash (program_id);

create table proposal_account_metas (
    proposal_address                varchar(48),
    program_id                      varchar(48),    
    pubkey                          varchar(48),
    is_signer                       bool                    not null,
    is_writable                     bool                    not null,
    primary key (proposal_address, program_id, pubkey)
);

create index if not exists proposal_account_proposal_address_idx on 
proposal_account_metas using hash (proposal_address);

create index if not exists proposal_account_proposal_program_id_idx on 
proposal_account_metas using hash (program_id);

create index if not exists proposal_account_proposal_pubkey_idx on 
proposal_account_metas using hash (pubkey);

create table proposal_metas (
    address                         varchar(48)             primary key,
    proposal                        varchar(48)             not null,
    title                           text                    not null,
    description_link                text                    not null
);

create index if not exists proposal_metas_proposal_idx on 
proposal_metas using hash (proposal);

create table votes (
    address                         varchar(48)             primary key,
    proposal                        varchar(48)             not null,
    voter                           varchar(48)             not null,
    bump                            smallint                not null,
    side                            smallint                not null,
    weight                          bigint                  not null
);

create index if not exists votes_proposal_idx on 
votes using hash (proposal);

create index if not exists votes_voter_idx on 
votes using hash (voter);

-- Goki Smart wallet program accounts tables

create table smart_wallets (
    address varchar(48) primary key,
    base varchar(48)             not null,
    bump smallint not null,
    threshold bigint not null,
    minimum_delay bigint not null,
    grace_period bigint not null,
    owner_set_seqno bigint not null,
    num_transactions bigint not null
);

create table smart_wallet_owners (
    smart_wallet_address varchar(48),
    owner_address varchar(48),
    index bigint not null,
    primary key (smart_wallet_address, owner_address)
);

create table transactions (
    address varchar(48)             primary key,
    smart_wallet varchar(48)             not null,
    index bigint not null,
    bump smallint not null,
    proposer varchar(48) not null,
    signers bool[] not null,
    owner_set_seqno bigint not null,
    eta bigint not null,
    executor varchar(48) not null,
    executed_at bigint not null
);

-- create table transaction_signers (
--     transaction_address varchar(48),
--     is_signer bool not null,
--     index bigint not null,
--     primary key (transaction_address, is_signer, index)
-- );

create table tx_instructions (
    transaction_address varchar(48) not null,
    program_id varchar(48) not null,
    data bytea not null,
    primary key (transaction_address, program_id)
);

create table tx_instruction_keys (
    transaction_address varchar(48) not null,
    program_id varchar(48) not null,
    pubkey varchar(48) not null,
    is_signer bool not null,
    is_writable bool not null,
    primary key (transaction_address, program_id, pubkey)
);

create table sub_account_infos (
    address                         varchar(48)             primary key,
    smart_wallet varchar(48),
    subaccount_type smallint,
    index bigint not null
);

create table instruction_buffers (
    address varchar(48) primary key,
    owner_set_seqno bigint not null,
    eta bigint not null,
    authority varchar(48) not null,
    executor varchar(48)             not null,
    smart_wallet varchar(48)             not null
);

create table ins_buffer_bundles (
    instruction_buffer_address varchar(48) primary key,
    is_executed bool not null
);

create table ins_buffer_bundle_instructions (
    instruction_buffer_address varchar(48),
    program_id                      varchar(48)             not null,
    data bytea not null,
    primary key (instruction_buffer_address, program_id)

);

create table ins_buffer_bundle_ins_keys (
    instruction_buffer_address varchar(48),
    program_id varchar(48) not null,
    pubkey varchar(48),
    is_signer bool not null,
    is_writable bool not null,
    primary key (instruction_buffer_address, pubkey)
);