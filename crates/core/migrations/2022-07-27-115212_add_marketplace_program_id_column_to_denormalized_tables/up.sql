alter table offers
add marketplace_program varchar(48);

alter table listings
add marketplace_program varchar(48);

alter table purchases
add marketplace_program varchar(48);

-- backfill marketplace_program column
update offers
set marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
where offers.auction_house = 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe';

update listings
set marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
where listings.auction_house = 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe';

update purchases
set marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
where purchases.auction_house = 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe';

update offers
set marketplace_program = 'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'
where offers.auction_house != 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe';

update listings
set marketplace_program = 'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'
where listings.auction_house != 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe';

update purchases
set marketplace_program = 'hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk'
where purchases.auction_house != 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe';

--add not null constraint
alter table offers alter column marketplace_program set not null;
alter table listings alter column marketplace_program set not null;
alter table purchases alter column marketplace_program set not null;