create table if not exists wallet_totals (
  address varchar(48) unique primary key not null,
  following bigint not null default 0,
  followers bigint not null default 0
);

create index if not exists wallet_totals_following on wallet_totals (following);
create index if not exists wallet_totals_followers on wallet_totals (followers desc);

insert into wallet_totals (address, following, followers)
 select
    address,
    (select count(from_account) from graph_connections WHERE from_account = wallets.address) as following,
    (select count(to_account) from graph_connections WHERE to_account = wallets.address) as follwers
    from ( select distinct on (address) address from (select from_account as address from graph_connections union select to_account as address from graph_connections) w) wallets;

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
