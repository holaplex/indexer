CREATE OR REPLACE FUNCTION floor_price_on_listing_update()
  RETURNS TRIGGER
  AS
$$
BEGIN
  INSERT INTO collection_stats (collection_address, floor_price)
  SELECT metadata_collection_keys.collection_address as collection_address, MIN(listings.price) AS floor_price
  FROM listings
  INNER JOIN metadatas ON(listings.metadata = metadatas.address)
  INNER JOIN metadata_collection_keys ON(metadatas.address = metadata_collection_keys.metadata_address)
  INNER JOIN auction_houses ON(listings.auction_house = auction_houses.address)
  WHERE metadatas.address = NEW.metadata
      AND auction_houses.treasury_mint = 'So11111111111111111111111111111111111111112'
      AND listings.purchase_id IS NULL
      AND listings.canceled_at IS NULL
      AND metadata_collection_keys.verified = true
  GROUP BY metadata_collection_keys.collection_address;
  RETURN NULL;
END;
$$ LANGUAGE PLPGSQL;

CREATE TRIGGER listing_added
  AFTER INSERT
  ON listings
  FOR EACH ROW
  EXECUTE PROCEDURE floor_price_on_listing_update();

CREATE TRIGGER listing_updated
  AFTER UPDATE
  ON listings
  FOR EACH ROW
  EXECUTE PROCEDURE floor_price_on_listing_update();