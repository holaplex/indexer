create table listing_denylist (
  listing_address varchar(48) primary key not null,
  hard_ban        boolean     not null
);
