create table collection_stats (
  collection_address  varchar(48) primary key,
  nft_count           bigint      not null,
  floor_price         bigint      null
);
