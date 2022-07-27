update offers
set auction_house = 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe'
where offers.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K';

update listings
set auction_house = 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe'
where listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K';

update purchases
set auction_house = 'E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe'
where purchases.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K';

alter table offers
drop column marketplace_program;

alter table listings
drop column marketplace_program;

alter table purchases
drop column marketplace_program;