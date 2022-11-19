create table accept_offer_ins (
    tx_signature  text primary key,
    buyer varchar(48) not null, 
    buyer_reward_token_account varchar(48) not null, 
    seller varchar(48) not null, 
    seller_reward_token_account varchar(48) not null, 
    offer varchar(48) not null, 
    token_account varchar(48) not null, 
    token_mint varchar(48) not null, 
    metadata varchar(48) not null, 
    treasury_mint varchar(48) not null, 
    seller_payment_receipt_account varchar(48) not null, 
    buyer_receipt_token_account varchar(48) not null, 
    authority varchar(48) not null, 
    escrow_payment_account varchar(48) not null, 
    auction_house varchar(48) not null, 
    auction_house_fee_account varchar(48) not null, 
    auction_house_treasury varchar(48) not null, 
    buyer_trade_state varchar(48) not null, 
    seller_trade_state varchar(48) not null, 
    free_seller_trade_state varchar(48) not null, 
    reward_center varchar(48) not null, 
    reward_center_reward_token_account varchar(48) not null, 
    ah_auctioneer_pda varchar(48) not null, 
    auction_house_program varchar(48) not null, 
    token_program varchar(48) not null, 
    escrow_payment_bump smallint not null, 
    free_trade_state_bump smallint not null,
    program_as_signer_bump smallint not null,
    seller_trade_state_bump smallint not null,
    buyer_trade_state_bump smallint not null,
    slot bigint not null
);

create table buy_listing_ins (
    tx_signature  text primary key,
    buyer varchar(48) not null,
    payment_account varchar(48) not null,
    transfer_authority varchar(48) not null,
    buyer_reward_token_account varchar(48) not null,
    seller varchar(48) not null,
    seller_reward_token_account varchar(48) not null,
    listing varchar(48) not null,
    token_account varchar(48) not null,
    token_mint varchar(48) not null,
    metadata varchar(48) not null,
    treasury_mint varchar(48) not null,
    seller_payment_receipt_account varchar(48) not null,
    buyer_receipt_token_account varchar(48) not null,
    authority varchar(48) not null,
    escrow_payment_account varchar(48) not null,
    auction_house varchar(48) not null,
    auction_house_fee_account varchar(48) not null,
    auction_house_treasury varchar(48) not null,
    buyer_trade_state varchar(48) not null,
    seller_trade_state varchar(48) not null,
    free_seller_trade_state varchar(48) not null,
    reward_center varchar(48) not null,
    reward_center_reward_token_account varchar(48) not null,
    ah_auctioneer_pda varchar(48) not null,
    auction_house_program varchar(48) not null,
    token_program varchar(48) not null,
    buyer_trade_state_bump smallint not null,
    escrow_payment_bump smallint not null,
    free_trade_state_bump smallint not null,
    seller_trade_state_bump smallint not null,
    program_as_signer_bump smallint not null,
    slot bigint not null
);

drop table if exists hpl_reward_center_execute_sale_ins;