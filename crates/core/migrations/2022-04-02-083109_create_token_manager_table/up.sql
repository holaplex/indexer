-- Your SQL goes here
create table token_managers (
    address                 varchar(48)     primary key,
    version                 smallint        not null,
    bump                    smallint        not null,
    count                   bigint          not null,
    num_invalidators        smallint        not null,
    issuer                  varchar(48)     not null,
    mint                    varchar(48)     not null,
    amount                  bigint          not null,
    kind                    smallint        not null,
    state                   smallint        not null,
    state_changed_at        timestamp       not null,
    invalidation_type       smallint        not null,
    recipient_token_account varchar(48)     not null,
    receipt_mint            varchar(48),
    claim_approver          varchar(48),
    transfer_authority      varchar(48)
);

create index if not exists token_managers_issuer_idx
on token_managers (issuer);

create table token_manager_invalidators (
    token_manager_address   varchar(48) not null,
    invalidator             varchar(48) not null,
    primary key (token_manager_address, invalidator),
    foreign key (token_manager_address) references token_managers (address)
);

create index if not exists token_manager_invalidators_idx
on token_manager_invalidators (token_manager_address);

create table time_invalidators (
    address                     varchar(48)     primary key,
    bump                        smallint        not null,
    token_manager_address       varchar(48)     not null,
    expiration                  timestamp,
    duration_seconds            bigint,
    extension_payment_amount    bigint,
    extension_duration_seconds  bigint,
    extension_payment_mint      varchar(48),
    max_expiration              timestamp,
    disable_partial_extension   boolean
);

create index if not exists time_invalidators_token_manager_idx
on time_invalidators (token_manager_address);

create table use_invalidators (
    address                     varchar(48)     primary key,
    bump                        smallint        not null,
    token_manager_address       varchar(48)     not null,
    usages                      bigint,
    use_authority               varchar(48),
    total_usages                bigint,
    extension_payment_amount    bigint,
    extension_payment_mint      varchar(48),
    extension_usages            bigint,
    max_usages                  bigint
);

create index if not exists use_invalidators_token_manager_idx
on use_invalidators (token_manager_address);
