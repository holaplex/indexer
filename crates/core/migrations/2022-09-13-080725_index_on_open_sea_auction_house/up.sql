create index if not exists listings_opensea_ah_idx on listings (auction_house)
where auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y';

create index offers_opensea_ah_idx on offers (auction_house)
where auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y';