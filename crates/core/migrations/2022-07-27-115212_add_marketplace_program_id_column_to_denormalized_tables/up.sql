alter table offers
add marketplace_program varchar(48) not null default 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K';

alter table listings
add marketplace_program varchar(48) not null default 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K';

alter table purchases
add marketplace_program varchar(48) not null default 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K';
