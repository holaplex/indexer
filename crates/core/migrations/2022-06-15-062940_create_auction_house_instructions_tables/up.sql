create table buy_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    wallet                                   varchar(48)     not null,
    payment_account                          varchar(48)     not null,
    transfer_authority                       varchar(48)     not null,
    treasury_mint                            varchar(48)     not null,
    token_account                            varchar(48)     not null,
    metadata                                 varchar(48)     not null,
    escrow_payment_account                   varchar(48)     not null,
    authority                                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    buyer_trade_state                        varchar(48)     not null,
    trade_state_bump                         smallint        not null,
    escrow_payment_bump                      smallint        not null,
    buyer_price                              bigint          not null,
    token_size                               bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table public_buy_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    wallet                                   varchar(48)     not null,
    payment_account                          varchar(48)     not null,
    transfer_authority                       varchar(48)     not null,
    treasury_mint                            varchar(48)     not null,
    token_account                            varchar(48)     not null,
    metadata                                 varchar(48)     not null,
    escrow_payment_account                   varchar(48)     not null,
    authority                                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    buyer_trade_state                        varchar(48)     not null,
    trade_state_bump                         smallint        not null,
    escrow_payment_bump                      smallint        not null,
    buyer_price                              bigint          not null,
    token_size                               bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table sell_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    wallet                                   varchar(48)     not null,
    token_account                            varchar(48)     not null,
    metadata                                 varchar(48)     not null,
    authority                                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    seller_trade_state                       varchar(48)     not null,
    free_seller_trader_state                 varchar(48)     not null,
    program_as_signer                        varchar(48)     not null,
    trade_state_bump                         smallint        not null,
    free_trade_state_bump                    smallint        not null,
    program_as_signer_bump                   smallint        not null,
    buyer_price                              bigint          not null,
    token_size                               bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table execute_sale_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    buyer                                    varchar(48)     not null,
    seller                                   varchar(48)     not null,
    token_account                            varchar(48)     not null,
    token_mint                               varchar(48)     not null,
    metadata                                 varchar(48)     not null,
    treasury_mint                            varchar(48)     not null,
    escrow_payment_account                   varchar(48)     not null,
    seller_payment_receipt_account           varchar(48)     not null,
    buyer_receipt_token_account              varchar(48)     not null,
    authority                                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    auction_house_treasury                   varchar(48)     not null,
    buyer_trade_state                        varchar(48)     not null,
    seller_trade_state                       varchar(48)     not null,
    free_trade_state                         varchar(48)     not null,
    program_as_signer                        varchar(48)     not null,
    escrow_payment_bump                      smallint        not null,
    free_trade_state_bump                    smallint        not null,
    program_as_signer_bump                   smallint        not null,
    buyer_price                              bigint          not null,
    token_size                               bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table cancel_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    wallet                                   varchar(48)     not null,
    token_account                            varchar(48)     not null,
    token_mint                               varchar(48)     not null,
    authority                                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    trade_state                              varchar(48)     not null,
    buyer_price                              bigint          not null,
    token_size                               bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table deposit_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    wallet                                   varchar(48)     not null,
    payment_account                          varchar(48)     not null,
    transfer_authority                       varchar(48)     not null,
    escrow_payment_account                   varchar(48)     not null,
    treasury_mint                            varchar(48)     not null,
    authority                                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    escrow_payment_bump                      smallint        not null,
    amount                                   bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table withdraw_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    wallet                                   varchar(48)     not null,
    receipt_account                          varchar(48)     not null,
    escrow_payment_account                   varchar(48)     not null,
    treasury_mint                            varchar(48)     not null,
    authority                                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    escrow_payment_bump                      smallint        not null,
    amount                                   bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table withdraw_from_fee_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    authority                                varchar(48)     not null,
    fee_withdrawal_destination               varchar(48)     not null,
    auction_house_fee_account                varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    amount                                   bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create table withdraw_from_treasury_instructions (
    id                                       uuid            primary key default gen_random_uuid(),
    treasury_mint                            varchar(48)     not null,
    authority                                varchar(48)     not null,
    treasury_withdrawal_destination          varchar(48)     not null,
    auction_house_treasury                   varchar(48)     not null,
    auction_house                            varchar(48)     not null,
    amount                                   bigint          not null,
    created_at                               timestamp       not null,
    slot                                     bigint          not null
);

create function check_slot() returns trigger
  language plpgsql
  as $EOF$
begin
  if old.slot > new.slot then
    return old;
  end if;

  return new;
end
$EOF$;

create trigger buy_ins_check_slot
before update on buy_instructions for row
execute function check_slot();

create trigger public_buy_ins_check_slot
before update on public_buy_instructions for row
execute function check_slot();

create trigger sell_ins_check_slot
before update on sell_instructions for row
execute function check_slot();

create trigger cancel_ins_check_slot
before update on cancel_instructions for row
execute function check_slot();

create trigger execute_sale_ins_check_slot
before update on execute_sale_instructions for row
execute function check_slot();

create trigger deposit_ins_check_slot
before update on deposit_instructions for row
execute function check_slot();

create trigger withdraw_from_fee_ins_check_slot
before update on withdraw_from_fee_instructions for row
execute function check_slot();

create trigger withdraw_from_treasury_ins_check_slot
before update on withdraw_from_treasury_instructions for row
execute function check_slot();

create trigger withdraw_ins_check_slot
before update on withdraw_instructions for row
execute function check_slot();