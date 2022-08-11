create type governanceaccounttype as enum (
  'Uninitialized', 'RealmV1', 'TokenOwnerRecordV1', 
  'GovernanceV1', 'ProgramGovernanceV1', 
  'ProposalV1', 'SignatoryRecordV1', 
  'VoteRecordV1', 'ProposalInstructionV1', 
  'MintGovernanceV1', 'TokenGovernanceV1', 
  'RealmConfig', 'VoteRecordV2', 'ProposalTransactionV2', 
  'ProposalV2', 'ProgramMetadata', 
  'RealmV2', 'TokenOwnerRecordV2', 
  'GovernanceV2', 'ProgramGovernanceV2', 
  'MintGovernanceV2', 'TokenGovernanceV2', 
  'SignatoryRecordV2'
);

create type votethresholdtype 
as enum ('YesVote', 'Quorum');

create type votetipping 
as enum ('Strict', 'Early', 'Disabled');        

create type mintmaxvotetype 
as enum ('SupplyFraction', 'Absolute');  

create type vote_record_v2_vote 
as enum ('Approve', 'Deny', 'Abstain', 'Veto');  

create type proposalstate 
as enum ('Draft', 'SigningOff', 'Voting', 'Succeeded', 'Executing', 'Completed', 'Cancelled', 'Defeated', 'ExecutingWithErrors');  

create type instructionexecutionflags 
as enum ('None', 'Ordered', 'UseTransaction');  

create type proposalvotetype 
as enum ('SingleChoice', 'MultiChoice');  

create type optionvoteresult
as enum ('None', 'Succeeded', 'Defeated');  

create table if not exists governances (
  address               varchar(48)                primary key,
  account_type          governanceaccounttype      not null,
  realm                 varchar(48)                not null,
  governed_account      varchar(48)                not null,
  proposals_count       bigint                     not null,
  reserved              bytea                      not null,
  voting_proposal_count smallint not null,
  slot                  bigint                     not null,
  write_version         bigint                     not null
);

create table if not exists governance_configs (
  governance_address                        varchar(48)                 primary key,
  vote_threshold_type                       votethresholdtype           not null,
  vote_threshold_percentage                 smallint                    not null,  
  min_community_weight_to_create_proposal   bigint                      not null,
  min_instruction_hold_up_time              bigint                      not null,
  max_voting_time                           bigint                      not null,
  vote_tipping                              votetipping                 not null,
  proposal_cool_off_time                    bigint                      not null,
  min_council_weight_to_create_proposal     bigint                      not null,
  slot                                      bigint                      not null,
  write_version                             bigint                      not null,

  foreign key (governance_address) references governances (address)
);

create table if not exists realms (
    address                                 varchar(48)                primary key,
    account_type                            governanceaccounttype      not null,
    community_mint                          varchar(48)                not null,
    reserved                                bytea                      not null,
    voting_proposal_count                   smallint                   not null,
    authority                               varchar(48),
    name                                    text                       not null,
    reserved_v2                             bytea                      not null,
    slot                                    bigint                     not null,
    write_version                           bigint                     not null
);

create table if not exists realm_configs (
    realm_address                                 varchar(48)           primary key,
    use_community_voter_weight_addin              bool                  not null,
    use_max_community_voter_weight_addin          bool                  not null,
    reserved                                      bytea                 not null,
    min_community_weight_to_create_governance     bigint                not null,
    community_mint_max_vote_weight_source         mintmaxvotetype       not null,
    community_mint_max_vote_weight                bigint                not null, 
    council_mint                                  varchar(48),
    slot                                          bigint                not null,
    write_version                                 bigint                not null,

    foreign key (realm_address) references realms (address)
);

create table if not exists vote_records_v2 (
    address                                 varchar(48)                primary key,
    account_type                            governanceaccounttype      not null,
    proposal                                varchar(48)                not null,
    governing_token_owner                   varchar(48)                not null,
    is_relinquished                         bool                       not null,
    voter_weight                            bigint                     not null,
    vote                                    vote_record_v2_vote        not null,  
    slot                                    bigint                     not null,
    write_version                           bigint                     not null
);

create table if not exists vote_record_v2_vote_approve_vote_choices (
    vote_record_v2_address                  varchar(48)                not null,
    rank                                    smallint                   not null,
    weight_percentage                       smallint                   not null,
    slot                                    bigint                     not null,
    write_version                           bigint                     not null,

    primary key (vote_record_v2_address, rank, weight_percentage)

);

create table if not exists token_owner_records_v2 (
    address                                 varchar(48)                 primary key,
    account_type                            governanceaccounttype       not null,
    realm                                   varchar(48)                 not null,
    governing_token_mint                    varchar(48)                 not null,
    governing_token_owner                   varchar(48)                 not null,
    governing_token_deposit_amount          bigint                      not null,
    unrelinquished_votes_count              bigint                      not null,
    total_votes_count                       bigint                      not null,
    outstanding_proposal_count              smallint                    not null,
    reserved                                bytea                       not null,
    governance_delegate                     varchar(48),
    slot                                    bigint                      not null,
    write_version                           bigint                      not null
);

create table if not exists signatory_records_v2 (
    address                                 varchar(48)                 primary key,
    account_type                            governanceaccounttype       not null,
    proposal                                varchar(48)                 not null,
    signatory                               varchar(48)                 not null,
    signed_off                              bool                        not null,
    slot                                    bigint                      not null,
    write_version                           bigint                      not null
);

create table if not exists proposals_v2 (
    address                                 varchar(48)                 primary key,
    account_type                            governanceaccounttype       not null,
    governance                              varchar(48)                 not null,
    governing_token_mint                    varchar(48)                 not null,
    state                                   proposalstate               not null,
    token_owner_record                      varchar(48)                 not null,
    signatories_count                       smallint                    not null,
    signatories_signed_off_count            smallint                    not null,
    vote_type                               proposalvotetype            not null,    
    deny_vote_weight                        bigint,
    veto_vote_weight                        bigint,
    abstain_vote_weight                     bigint,
    start_voting_at                         timestamp,
    draft_at                                timestamp                   not null,
    signing_off_at                          timestamp,
    voting_at                               timestamp,
    voting_at_slot                          bigint,
    voting_completed_at                     timestamp,
    executing_at                            timestamp,
    closed_at                               timestamp,
    execution_flags                         instructionexecutionflags   not null,
    max_vote_weight                         bigint,
    max_voting_time                         bigint,
    vote_threshold_type                     votethresholdtype,
    vote_threshold_percentage               smallint,
    name                                    text                        not null,
    description_link                        text                        not null,
    slot                                    bigint                      not null,
    write_version                           bigint                      not null
);

create table if not exists proposal_vote_type_multi_choices (
    address                                 varchar(48)                 primary key,
    max_voter_options                       smallint                    not null,
    max_winning_options                     smallint                    not null,
    slot                                    bigint                      not null,        
    write_version                           bigint                      not null
);

create table if not exists proposal_options (
    proposal_address                        varchar(48)                 not null,
    label                                   text                        not null,
    vote_weight                             bigint                      not null,
    vote_result                             optionvoteresult            not null,
    transactions_executed_count             smallint                    not null,
    transactions_count                      smallint                    not null,
    transactions_next_index                 smallint                    not null,
    slot                                    bigint                      not null,        
    write_version                           bigint                      not null,

    primary key (proposal_address, label)
);

create trigger governances_check_slot_wv
before update on governances for row
execute function check_slot_wv();

create trigger governance_configs_check_slot_wv
before update on governance_configs for row
execute function check_slot_wv();

create trigger realms_check_slot_wv
before update on realms for row
execute function check_slot_wv();

create trigger realm_configs_check_slot_wv
before update on realm_configs for row
execute function check_slot_wv();

create trigger vote_records_v2_check_slot_wv
before update on vote_records_v2 for row
execute function check_slot_wv();

create trigger vote_record_v2_vote_approve_vote_choices_check_slot_wv
before update on vote_record_v2_vote_approve_vote_choices for row
execute function check_slot_wv();

create trigger token_owner_records_v2_check_slot_wv
before update on token_owner_records_v2 for row
execute function check_slot_wv();

create trigger signatory_records_v2_check_slot_wv
before update on signatory_records_v2 for row
execute function check_slot_wv();

create trigger proposals_v2_check_slot_wv
before update on proposals_v2 for row
execute function check_slot_wv();

create trigger proposal_vote_type_multi_choices_check_slot_wv
before update on proposal_vote_type_multi_choices for row
execute function check_slot_wv();

create trigger proposal_options_check_slot_wv
before update on proposal_options for row
execute function check_slot_wv();