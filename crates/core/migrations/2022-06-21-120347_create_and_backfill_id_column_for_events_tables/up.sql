alter table offer_events 
add column offer_id uuid not null default '00000000-0000-0000-0000-000000000000';

alter table purchase_events 
add column purchase_id uuid not null default '00000000-0000-0000-0000-000000000000';

alter table listing_events 
add column listing_id uuid not null default '00000000-0000-0000-0000-000000000000';

update offer_events
set offer_id = o.id
from offer_events oe
inner join bid_receipts br on (oe.bid_receipt_address = br.address)
inner join offers o on ((o.trade_state = br.trade_state)
	and (o.metadata = br.metadata))
where offer_events.feed_event_id = oe.feed_event_id;

update purchase_events
set purchase_id = p.id
from purchase_events pe
inner join purchase_receipts pr on (pe.purchase_receipt_address = pr.address)
inner join purchases p on ((p.buyer = pr.buyer)
	and (p.seller = pr.seller)
	and (p.auction_house = pr.auction_house)
	and (p.metadata = pr.metadata)
	and (p.price = pr.price)
	and (p.token_size = pr.token_size))
where purchase_events.feed_event_id = pe.feed_event_id;

update listing_events
set listing_id = l.id
from listing_events le
inner join listing_receipts lr on (le.listing_receipt_address = lr.address)
inner join listings l on ((l.trade_state = lr.trade_state)
	and (l.metadata = lr.metadata))
where listing_events.feed_event_id = le.feed_event_id;

alter table offer_events 
drop column bid_receipt_address;

alter table purchase_events 
drop column purchase_receipt_address;

alter table listing_events
drop column listing_receipt_address;