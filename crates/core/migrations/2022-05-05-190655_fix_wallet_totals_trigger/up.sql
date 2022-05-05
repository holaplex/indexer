CREATE OR REPLACE FUNCTION wallet_totals_graph_connection_inserted()
  RETURNS TRIGGER
  AS
$$
BEGIN
  INSERT INTO wallet_totals (address, following, followers)
    VALUES (NEW.from_account, 1, 0)
    ON CONFLICT (address) DO UPDATE
      SET following = wallet_totals.following + 1;

  INSERT INTO wallet_totals (address, following, followers)
    VALUES (NEW.to_account, 0, 1)
    ON CONFLICT (address) DO UPDATE
      SET followers = wallet_totals.followers + 1;

  RETURN NULL;
END;
$$ LANGUAGE PLPGSQL;
