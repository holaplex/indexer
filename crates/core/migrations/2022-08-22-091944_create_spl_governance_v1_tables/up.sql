create type voteweightv1 
as enum ('Yes', 'No');

create table if not exists vote_records_v1 (
    address                                 varchar(48)                 primary key,
    account_type                            governanceaccounttype       not null,
    proposal                                varchar(48)                 not null,
    governing_token_owner                   varchar(48)                 not null,
    is_relinquished                         bool                        not null,
    vote_type                               voteweightv1                not null,
    vote_weight                             bigint                      not null,
    slot                                    bigint                      not null,
    write_version                           bigint                      not null
);

create table if not exists proposals_v1 (
    address                                 varchar(48)                 primary key,
    account_type                            governanceaccounttype       not null,
    governance                              varchar(48)                 not null,
    governing_token_mint                    varchar(48)                 not null,
    state                                   proposalstate               not null,
    token_owner_record                      varchar(48)                 not null,
    signatories_count                       smallint                    not null,
    signatories_signed_off_count            smallint                    not null,
    yes_votes_count                         bigint                      not null,
    no_votes_count                          bigint                      not null,
    instructions_executed_count             smallint not null,
    instructions_count                      smallint not null,
    instructions_next_index                 smallint not null,
    draft_at                                timestamp                   not null,
    signing_off_at                          timestamp,
    voting_at                               timestamp,
    voting_at_slot                          bigint,
    voting_completed_at                     timestamp,
    executing_at                            timestamp,
    closed_at                               timestamp,
    execution_flags                         instructionexecutionflags   not null,
    max_vote_weight                         bigint,
    vote_threshold_type                     votethresholdtype,
    vote_threshold_percentage               smallint,
    name                                    text                        not null,
    description_link                        text                        not null,
    slot                                    bigint                      not null,
    write_version                           bigint                      not null
);

create trigger vote_records_v1_check_slot_wv
before update on vote_records_v1 for row
execute function check_slot_wv();

create trigger proposals_v1_check_slot_wv
before update on proposals_v1 for row
execute function check_slot_wv();

alter table governance_configs
alter column min_community_weight_to_create_proposal type numeric;

alter table realm_configs
alter column min_community_weight_to_create_governance type numeric;

alter table signatory_records_v2
rename to signatory_records;

alter table token_owner_records_v2
rename to token_owner_records;