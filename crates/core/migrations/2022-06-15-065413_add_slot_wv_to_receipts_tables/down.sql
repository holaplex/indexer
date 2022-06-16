alter table listing_receipts
drop column slot,
drop column write_version;

alter table bid_receipts
drop column slot,
drop column write_version;

alter table purchase_receipts
drop column slot,
drop column write_version;

drop trigger listing_receipts_check_slot_wv on listing_receipts;
drop trigger bid_receipts_check_slot_wv on bid_receipts;
drop trigger purchase_receipts_check_slot_wv on purchase_receipts;
