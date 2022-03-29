alter table token_accounts
drop column slot;
drop trigger set_token_account_updated_at on token_accounts;
drop function trigger_set_updated_at_timestamp();