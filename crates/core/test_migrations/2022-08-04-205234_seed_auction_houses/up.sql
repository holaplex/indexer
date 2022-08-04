insert into auction_houses values (
  'ah_address',                                   -- address
  'So11111111111111111111111111111111111111112',  -- treasury_mint
  'ah_treasury',                                  -- auction_house_treasury
  'ah_withdrawal',                                -- treasury_withdrawal_destination
  'ah_fee_withdrawal',                            -- fee_withdrawal_destination
  'ah_authority',                                 -- authority
  'ah_creator',                                   -- creator
  0,                                              -- bump
  0,                                              -- treasury_bump
  0,                                              -- fee_payer_bump
  1000,                                           -- seller_fee_basis_points
  true,                                           -- requires_sign_off
  false,                                          -- can_change_sale_price
  'ah_fees'                                       -- auction_house_fee_account
);

insert into listings values (
  '00000000-0000-0000-0001-000000000000', -- id
  'meta_trade_state_0',                   -- trade_state
  'ah_address',                           -- auction_house
  'meta_owner',                           -- seller
  'meta_address_0',                       -- metadata
  null,                                   -- purchase_id
  1,                                      -- price
  1,                                      -- token_size
  0,                                      -- trade_state_bump
  '2020-01-01',                           -- created_at
  null,                                   -- canceled_at
  1,                                      -- slot
  0,                                      -- write_version
  'market_program',                       -- marketplace_program
  null                                    -- expiry
), (
  '00000000-0000-0000-0001-000000000001', -- id
  'meta_trade_state_1',                   -- trade_state
  'ah_address',                           -- auction_house
  'meta_owner',                           -- seller
  'meta_address_1',                       -- metadata
  null,                                   -- purchase_id
  1,                                      -- price
  1,                                      -- token_size
  0,                                      -- trade_state_bump
  '2020-01-01',                           -- created_at
  null,                                   -- canceled_at
  1,                                      -- slot
  0,                                      -- write_version
  'market_program',                       -- marketplace_program
  null                                    -- expiry
), (
  '00000000-0000-0000-0001-000000000002', -- id
  'meta_trade_state_2',                   -- trade_state
  'ah_address',                           -- auction_house
  'meta_owner',                           -- seller
  'meta_address_2',                       -- metadata
  null,                                   -- purchase_id
  1,                                      -- price
  1,                                      -- token_size
  0,                                      -- trade_state_bump
  '2020-01-01',                           -- created_at
  null,                                   -- canceled_at
  1,                                      -- slot
  0,                                      -- write_version
  'market_program',                       -- marketplace_program
  null                                    -- expiry
), (
  '00000000-0000-0000-0001-000000000003', -- id
  'meta_trade_state_3',                   -- trade_state
  'ah_address',                           -- auction_house
  'meta_owner',                           -- seller
  'meta_address_3',                       -- metadata
  '00000000-0000-0000-0002-000000000003', -- purchase_id
  1,                                      -- price
  1,                                      -- token_size
  0,                                      -- trade_state_bump
  '2020-01-01',                           -- created_at
  null,                                   -- canceled_at
  1,                                      -- slot
  0,                                      -- write_version
  'market_program',                       -- marketplace_program
  null                                    -- expiry
);

insert into purchases values (
  '00000000-0000-0000-0002-000000000003', -- id
  'meta_buyer',                           -- buyer
  'meta_owner',                           -- seller
  'ah_address',                           -- auction_house
  'meta_address_3',                       -- metadata
  1,                                      -- token_size
  1,                                      -- price
  '2020-01-01',                           -- created_at
  1,                                      -- slot
  0,                                      -- write_version
  'market_program'                        -- marketplace_program
);
