alter table listings
alter column total_uncancelled_bids set default 0,
alter column total_uncancelled_bids drop not null;

alter table metadatas
alter column primary_sale_happened  drop not null,
alter column is_mutable             drop not null;

alter table metadata_creators
alter column verified drop not null;
