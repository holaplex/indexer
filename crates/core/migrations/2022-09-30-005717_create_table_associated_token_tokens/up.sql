CREATE TABLE associated_token_accounts (
    address varchar(48) PRIMARY KEY,
    mint varchar(48) NOT NULL,
    owner varchar(48) NOT NULL,
    amount bigint NOT NULL,
    slot bigint NOT NULL,
    write_version bigint NOT null
);

CREATE INDEX IF NOT EXISTS ata_mint_idx ON associated_token_accounts (mint);

CREATE INDEX IF NOT EXISTS ata_owner_idx ON associated_token_accounts (OWNER);

create trigger associated_token_accounts_check_slot_wv
before update on associated_token_accounts for row
execute function check_slot_wv();