alter table listings
alter column total_uncancelled_bids drop default,
alter column total_uncancelled_bids drop not null; -- redundant, just in case

alter table metadatas
alter column primary_sale_happened  set not null,
alter column is_mutable             set not null;

alter table metadata_creators
alter column verified set not null;
