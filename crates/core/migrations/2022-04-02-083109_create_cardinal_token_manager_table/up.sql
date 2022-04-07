create table cardinal_token_managers (
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

create index if not exists cardinal_token_managers_issuer_idx
on cardinal_token_managers (issuer);

create table cardinal_token_manager_invalidators (
    token_manager_address   varchar(48) not null,
    invalidator             varchar(48) not null,
    primary key (token_manager_address, invalidator),
    foreign key (token_manager_address) references cardinal_token_managers (address)
);

create index if not exists cardinal_token_manager_invalidators_idx
on cardinal_token_manager_invalidators (token_manager_address);

create table cardinal_time_invalidators (
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

create index if not exists cardinal_time_invalidators_token_manager_idx
on cardinal_time_invalidators (token_manager_address);

create table cardinal_use_invalidators (
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

create index if not exists cardinal_use_invalidators_token_manager_idx
on cardinal_use_invalidators (token_manager_address);

create table cardinal_paid_claim_approvers (
    address                     varchar(48)     primary key,
    bump                        smallint        not null,
    token_manager_address       varchar(48)     not null,
    payment_amount              bigint          not null,
    payment_mint                varchar(48)     not null,
    collector                   varchar(48)     not null
);

create index if not exists cardinal_paid_claim_approver_token_manager_idx
on cardinal_paid_claim_approvers (token_manager_address);

create table carindal_claim_events (
    id                          SERIAL          PRIMARY KEY,
    token_manager_address       varchar(48)     not null,
    version                     smallint        not null,
    bump                        smallint        not null,
    count                       bigint          not null,
    num_invalidators            smallint        not null,
    issuer                      varchar(48)     not null,
    mint                        varchar(48)     not null,
    amount                      bigint          not null,
    kind                        smallint        not null,
    state                       smallint        not null,
    state_changed_at            timestamp       not null,
    invalidation_type           smallint        not null,
    recipient_token_account     varchar(48)     not null,
    receipt_mint                                varchar(48),
    claim_approver                              varchar(48),
    transfer_authority                          varchar(48),
    invalidators                                varchar(48)     ARRAY,
    payment_amount                              bigint,
    payment_mint                                varchar(48),
    time_invalidator_expiration                 timestamp,
    time_invalidator_duration_seconds           bigint,
    time_invalidator_extension_payment_amount   bigint,
    time_invalidator_extension_duration_seconds bigint,
    time_invalidator_extension_payment_mint     varchar(48),
    time_invalidator_max_expiration             timestamp,
    time_invalidator_disable_partial_extension  boolean,
    use_invalidator_usages                      bigint,
    use_invalidator_use_authority               varchar(48),
    use_invalidator_total_usages                bigint,
    use_invalidator_extension_payment_amount    bigint,
    use_invalidator_extension_payment_mint      varchar(48),
    use_invalidator_extension_usages            bigint,
    use_invalidator_max_usages                  bigint
);
