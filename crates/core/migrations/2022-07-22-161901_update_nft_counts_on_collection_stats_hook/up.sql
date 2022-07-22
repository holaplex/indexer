CREATE OR REPLACE FUNCTION nft_count_on_nft_added()
  RETURNS TRIGGER
  AS
$$
BEGIN
  INSERT INTO collection_stats (collection_address, nft_count)
    VALUES (NEW.collection_address, 1)
    ON CONFLICT (collection_address) DO UPDATE
      SET nft_count = collection_stats.nft_count + 1;

  RETURN NULL;
END;
$$ LANGUAGE PLPGSQL;

CREATE TRIGGER nft_collection_key_added
  AFTER INSERT
  ON metadata_collection_keys
  FOR EACH ROW
  EXECUTE PROCEDURE nft_count_on_nft_added();
