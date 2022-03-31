create index if not exists meta_name on
metadatas using btree (name);

create index if not exists mcvrfd_idx on 
metadata_creators using btree (creator_address, verified) where (verified = true);

create index if not exists maddy_attrs on 
attributes using btree (metadata_address);

create index if not exists coll_name_metadata_address_idx on 
metadata_collections using btree (name, metadata_address);

create index if not exists tok_amt_idx on 
token_accounts using btree (amount);

create index if not exists tok_mint_idx on 
token_accounts using btree (mint_address);

create index if not exists tok_own_idx on 
token_accounts using btree (owner_address);

create index if not exists token_account_amt_mint_idx on 
token_accounts using btree (mint_address, amount) where (amount = 1);
