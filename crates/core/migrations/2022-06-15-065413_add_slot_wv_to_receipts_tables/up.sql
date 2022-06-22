alter table listing_receipts
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;

alter table bid_receipts
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;

alter table purchase_receipts
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;


create trigger listing_receipts_check_slot_wv
before update on listing_receipts for row
execute function check_slot_wv();

create trigger bid_receipts_check_slot_wv
before update on bid_receipts for row
execute function check_slot_wv();

create trigger purchase_receipts_check_slot_wv
before update on purchase_receipts for row
execute function check_slot_wv();