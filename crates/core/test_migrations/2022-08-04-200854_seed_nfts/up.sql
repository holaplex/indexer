insert into metadatas values (
  'meta_address_0',             -- address
  'Metadata 0',                 -- name
  'NFT',                        -- symbol
  'https://example.com/0.json', -- uri
  1000,                         -- seller_fee_basis_points
  'updater',                    -- update_authority_address
  'mint_0',                     -- mint_address
  false,                        -- primary_sale_happened
  false,                        -- is_mutable
  1337,                         -- edition_nonce
  'edition',                    -- edition_pda
  'NonFungible',                -- token_standard
  1,                            -- slot
  false                         -- burned
), (
  'meta_address_1',             -- address
  'Metadata 1',                 -- name
  'NFT',                        -- symbol
  'https://example.com/1.json', -- uri
  1000,                         -- seller_fee_basis_points
  'updater',                    -- update_authority_address
  'mint_1',                     -- mint_address
  false,                        -- primary_sale_happened
  false,                        -- is_mutable
  1337,                         -- edition_nonce
  'edition',                    -- edition_pda
  'NonFungible',                -- token_standard
  1,                            -- slot
  false                         -- burned
), (
  'meta_address_2',             -- address
  'Metadata 2',                 -- name
  'NFT',                        -- symbol
  'https://example.com/2.json', -- uri
  1000,                         -- seller_fee_basis_points
  'updater',                    -- update_authority_address
  'mint_2',                     -- mint_address
  false,                        -- primary_sale_happened
  false,                        -- is_mutable
  1337,                         -- edition_nonce
  'edition',                    -- edition_pda
  'NonFungible',                -- token_standard
  1,                            -- slot
  false                         -- burned
), (
  'meta_address_3',             -- address
  'Metadata 3',                 -- name
  'NFT',                        -- symbol
  'https://example.com/3.json', -- uri
  1000,                         -- seller_fee_basis_points
  'updater',                    -- update_authority_address
  'mint_3',                     -- mint_address
  false,                        -- primary_sale_happened
  false,                        -- is_mutable
  1337,                         -- edition_nonce
  'edition',                    -- edition_pda
  'NonFungible',                -- token_standard
  1,                            -- slot
  false                         -- burned
), (
  'collection_address',         -- address
  'Collection',                 -- name
  'COLL',                       -- symbol
  'https://example.com/c.json', -- uri
  1000,                         -- seller_fee_basis_points
  'updater',                    -- update_authority_address
  'collection_mint',            -- mint_address
  false,                        -- primary_sale_happened
  false,                        -- is_mutable
  1337,                         -- edition_nonce
  'Edition',                    -- edition_pda
  'NonFungible',                -- token_standard
  1,                            -- slot
  false                         -- burned
);

insert into metadata_jsons values (
  'meta_address_0',             -- metadata_address
  '00',                         -- fingerprint
  '2020-01-01',                 -- updated_at
  'An NFT',                     -- description
  'https://example.com/0.jpg',  -- image
  'https://example.com/0.mkv',  -- animation_url
  'https://example.com/0',      -- external_url
  'test',                       -- category
  '{}',                         -- raw_content
  'seeded',                     -- model
  'https://example.com/0.json', -- fetch_uri
  1,                            -- slot
  0                             -- write_version
), (
  'meta_address_1',             -- metadata_address
  '01',                         -- fingerprint
  '2020-01-01',                 -- updated_at
  'An NFT',                     -- description
  'https://example.com/1.jpg',  -- image
  'https://example.com/1.mkv',  -- animation_url
  'https://example.com/1',      -- external_url
  'test',                       -- category
  '{}',                         -- raw_content
  'seeded',                     -- model
  'https://example.com/1.json', -- fetch_uri
  1,                            -- slot
  0                             -- write_version
), (
  'meta_address_2',             -- metadata_address
  '02',                         -- fingerprint
  '2020-01-01',                 -- updated_at
  'An NFT',                     -- description
  'https://example.com/2.jpg',  -- image
  'https://example.com/2.mkv',  -- animation_url
  'https://example.com/2',      -- external_url
  'test',                       -- category
  '{}',                         -- raw_content
  'seeded',                     -- model
  'https://example.com/2.json', -- fetch_uri
  1,                            -- slot
  0                             -- write_version
), (
  'meta_address_3',             -- metadata_address
  '03',                         -- fingerprint
  '2020-01-01',                 -- updated_at
  'An NFT',                     -- description
  'https://example.com/3.jpg',  -- image
  'https://example.com/3.mkv',  -- animation_url
  'https://example.com/3',      -- external_url
  'test',                       -- category
  '{}',                         -- raw_content
  'seeded',                     -- model
  'https://example.com/3.json', -- fetch_uri
  1,                            -- slot
  0                             -- write_version
), (
  'collection_address',         -- metadata_address
  'c0',                         -- fingerprint
  '2020-01-01',                 -- updated_at
  'An NFT collection',          -- description
  'https://example.com/c.jpg',  -- image
  'https://example.com/c.mkv',  -- animation_url
  'https://example.com/c',      -- external_url
  'test',                       -- category
  '{}',                         -- raw_content
  'seeded',                     -- model
  'https://example.com/c.json', -- fetch_uri
  1,                            -- slot
  0                             -- write_version
);

insert into metadata_collection_keys values (
  'meta_address_0',     -- metadata_address
  'collection_mint',    -- collection_address
  true                  -- verified
), (
  'meta_address_1',     -- metadata_address
  'collection_mint',    -- collection_address
  true                  -- verified
), (
  'meta_address_2',     -- metadata_address
  'collection_mint',    -- collection_address
  true                  -- verified
), (
  'meta_address_3',     -- metadata_address
  'collection_mint',    -- collection_address
  true                  -- verified
);

insert into current_metadata_owners values (
  'mint_0',     -- mint_address
  'meta_owner', -- owner_address
  'ata_owner',  -- token_account_address
  '2020-01-01', -- updated_at
  1             -- slot
), (
  'mint_1',     -- mint_address
  'meta_owner', -- owner_address
  'ata_owner',  -- token_account_address
  '2020-01-01', -- updated_at
  1             -- slot
), (
  'mint_2',     -- mint_address
  'meta_owner', -- owner_address
  'ata_owner',  -- token_account_address
  '2020-01-01', -- updated_at
  1             -- slot
), (
  'mint_3',     -- mint_address
  'meta_owner', -- owner_address
  'ata_owner',  -- token_account_address
  '2020-01-01', -- updated_at
  1             -- slot
), (
  'collection_mint',  -- mint_address
  'meta_owner',       -- owner_address
  'ata_owner',        -- token_account_address
  '2020-01-01',       -- updated_at
  1                   -- slot
);
