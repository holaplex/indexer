create table if not exists wallet_totals as
  select 
    from_account as address, 
    (select count(from_account) from graph_connections WHERE from_account = wallets.from_account) as following,
    (select count(to_account) from graph_connections WHERE to_account = wallets.from_account) as follwers
    from (select distinct on (from_account) from_account from graph_connections) wallets;

ALTER TABLE wallet_totals ADD PRIMARY KEY (address);

CREATE OR REPLACE FUNCTION wallet_totals_graph_connection_inserted()
  RETURNS TRIGGER 
  AS
$$
BEGIN
  INSERT INTO wallet_totals (address, following, followers) 
    VALUES (NEW.from_account, 1, 0)
    ON CONFLICT (address) DO UPDATE 
      SET following = following + 1;
  INSERT INTO wallet_totals (address, following, followers) 
    VALUES (NEW.to_account, 0, 1)
    ON CONFLICT (address) DO UPDATE 
      SET followers = followers + 1;

	RETURN NULL;
END;
$$ LANGUAGE PLPGSQL;

CREATE TRIGGER wallet_totals_graph_connection_inserted
  AFTER INSERT
  ON graph_connections
  FOR EACH ROW
  EXECUTE PROCEDURE wallet_totals_graph_connection_inserted();