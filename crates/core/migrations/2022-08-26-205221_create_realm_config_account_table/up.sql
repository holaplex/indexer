create table if not exists realm_config_accounts (
    address                                 varchar(48)                 primary key,
    account_type                            governanceaccounttype       not null,
    realm                                   varchar(48)                 not null,
    community_voter_weight_addin            varchar(48),
    max_community_voter_weight_addin        varchar(48),
    council_voter_weight_addin              varchar(48),
    council_max_vote_weight_addin           varchar(48),
    slot                                    bigint                      not null,
    write_version                           bigint                      not null
);


create trigger realm_config_accounts_check_slot_wv
before update on realm_config_accounts for row
execute function check_slot_wv();
