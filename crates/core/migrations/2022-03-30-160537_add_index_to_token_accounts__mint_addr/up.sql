create index if not exists token_account_mint_addr_idx
on token_accounts using hash (mint_address);