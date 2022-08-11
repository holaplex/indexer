create type transactionexecutionstatus 
as enum ('None', 'Success', 'Error');

create table if not exists proposal_transactions (
    address                                 varchar(48)                 primary key,
    account_type                            governanceaccounttype       not null,
    proposal                                varchar(48)                 not null,
    option_index                            smallint                    not null,
    transaction_index                       smallint                    not null,
    hold_up_time                            bigint                      not null,
    executed_at                             timestamp                   null,
    execution_status                        transactionexecutionstatus  not null,
    slot                                    bigint                      not null,
    write_version                           bigint                      not null
);

create table if not exists proposal_transaction_instructions (
    proposal_transaction                    varchar(48)                 not null,
    program_id                              varchar(48)                 not null,
    data                                    bytea                       not null,
    slot                                    bigint                      not null,
    write_version                           bigint                      not null,

    primary key (proposal_transaction, program_id, data)
);

create table if not exists proposal_transaction_instruction_accounts (
    proposal_transaction                    varchar(48)                 not null,
    account_pubkey                          varchar(48)                 not null,
    is_signer                               bool                        not null,
    is_writable                             bool                        not null,
    slot                                    bigint                      not null,
    write_version                           bigint                      not null,

    primary key (proposal_transaction, account_pubkey)
);

create trigger proposal_transactions_check_slot_wv
before update on proposal_transactions for row
execute function check_slot_wv();

create trigger proposal_transaction_instructions_check_slot_wv
before update on proposal_transaction_instructions for row
execute function check_slot_wv();

create trigger proposal_transaction_instruction_accounts_check_slot_wv
before update on proposal_transaction_instruction_accounts for row
execute function check_slot_wv();