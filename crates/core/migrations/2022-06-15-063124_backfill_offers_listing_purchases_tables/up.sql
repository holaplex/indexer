insert into purchases (buyer, seller, auction_house, metadata, token_size, price, created_at)
select buyer, seller, auction_house, metadata, token_size, price, created_at
from purchase_receipts;

insert into offers (trade_state, auction_house, buyer, metadata, token_account, price, token_size, trade_state_bump, created_at, canceled_at)
select trade_state, auction_house, buyer, metadata, token_account, price, token_size, trade_state_bump, created_at, canceled_at
from bid_receipts;

insert into listings (trade_state, auction_house, seller, metadata, price, token_size, trade_state_bump, created_at, canceled_at)
select trade_state, auction_house, seller, metadata, price, token_size, trade_state_bump, created_at, canceled_at
from listing_receipts;

update offers
set purchase_id = purchases.id
from offers o
inner join purchases
  on ((o.auction_house = purchases.auction_house)
  and (o.buyer = purchases.buyer)
  and (o.metadata = purchases.metadata)
  and (o.token_size = purchases.token_size)
  and (o.price = purchases.price)
  )
where o.purchase_id is null
and o.canceled_at is null
and offers.id = o.id;

update listings
set purchase_id = purchases.id
from listings l
inner join purchases
  on ((l.auction_house = purchases.auction_house)
  and (l.seller = purchases.seller)
  and (l.metadata = purchases.metadata)
  and (l.price = purchases.price)
  and (l.token_size = purchases.token_size)
  )
where l.purchase_id is null
and l.canceled_at is null
and listings.id = l.id;