delete from auction_houses;

alter table auction_houses
add  auction_house_fee_account varchar(48) not null;