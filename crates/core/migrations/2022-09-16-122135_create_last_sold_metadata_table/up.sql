CREATE TABLE IF NOT EXISTS LAST_SOLD_METADATAS AS
	(SELECT METADATA, ID AS PURCHASE_ID, PRICE, CREATED_AT
		FROM
			(SELECT PURCHASES.*,
					ROW_NUMBER() OVER(PARTITION BY METADATA ORDER BY CREATED_AT DESC) TOPROW
				FROM PURCHASES) p
		WHERE P.TOPROW = 1 );

alter table last_sold_metadatas add primary key (metadata);
alter table last_sold_metadatas add unique  (purchase_id);

CREATE OR REPLACE FUNCTION INSERT_LAST_SOLD_METADATA() RETURNS TRIGGER AS $BODY$
BEGIN
    INSERT INTO
        last_sold_metadatas(metadata, purchase_id, price, created_at)
        VALUES(new.metadata,new.id, new.price, new.created_at)
		on conflict (metadata) do update set metadata = new.metadata, purchase_id = new.id, price = new.price, created_at = new.created_at;
           RETURN new;
END;
$BODY$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION UPDATE_OLDER_PURCHASE_ONLY() RETURNS TRIGGER LANGUAGE PLPGSQL AS $EOF$
begin
  if (old.created_at > new.created_at) then
    return old;
  end if;

  return new;
end
$EOF$;

CREATE OR REPLACE TRIGGER UPDATE_OLDER_PURCHASE_ONLY_TRIGGER BEFORE
UPDATE ON last_sold_metadatas
FOR ROW EXECUTE PROCEDURE UPDATE_OLDER_PURCHASE_ONLY();

CREATE OR REPLACE TRIGGER INSERT_LAST_SOLD_METADATA_TRIGGER AFTER
INSERT ON PURCHASES
FOR EACH ROW EXECUTE PROCEDURE INSERT_LAST_SOLD_METADATA();