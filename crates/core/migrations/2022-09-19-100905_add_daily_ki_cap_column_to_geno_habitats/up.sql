ALTER TABLE GENO_HABITAT_DATAS
    ADD COLUMN daily_ki_harvesting_cap numeric NOT NULL DEFAULT 0,
    ADD COLUMN KI_AVAILABLE_TO_HARVEST numeric GENERATED ALWAYS AS (daily_ki_harvesting_cap - KI_HARVESTED) STORED,
    ADD COLUMN HAS_MAX_KI BOOL GENERATED ALWAYS AS (daily_ki_harvesting_cap = KI_HARVESTED) STORED;

CREATE INDEX GENO_HABITAT_DATAS_HABITAT_MINT_IDX ON GENO_HABITAT_DATAS (HABITAT_MINT);

DO $$
BEGIN
  	IF (SELECT NOT EXISTS (SELECT COUNT(*) FROM geno_habitat_datas where daily_ki_harvesting_cap != 0)) THEN
	
    update geno_habitat_datas set daily_ki_harvesting_cap = a.daily_ki_harvesting_cap
from (SELECT habitat,
                CASE WHEN COUNT(HABITAT) > 1 THEN
                    SUM(HABITAT_CAP) * 1.1
                ELSE
                    SUM(HABITAT_CAP)
                END AS daily_ki_harvesting_cap
            FROM (
                SELECT
                    HABITAT_MINT AS HABITAT,
                    CASE WHEN LEVEL = 1 THEN
                        10000000000 * GENESIS::int + 30000000000
                    WHEN LEVEL = 2 THEN
                        10000000000 * GENESIS::int + 50000000000
                    WHEN LEVEL = 3 THEN
                        10000000000 * GENESIS::int + 70000000000
                    ELSE
                        0
                    END AS HABITAT_CAP
                FROM
                    GENO_HABITAT_DATAS
                WHERE
				parent_habitat is null and 
                    is_sub_habitat = FALSE
                UNION ALL
                SELECT
                    PARENT_HABITAT AS HABITAT,
                    CASE WHEN LEVEL = 1 THEN
                        10000000000 * GENESIS::int + 30000000000
                    WHEN LEVEL = 2 THEN
                        10000000000 * GENESIS::int + 50000000000
                    WHEN LEVEL = 3 THEN
                        10000000000 * GENESIS::int + 70000000000
                    ELSE
                        0
                    END AS HABITAT_CAP
                FROM
                    GENO_HABITAT_DATAS
                WHERE
                    PARENT_HABITAT is not null
                    AND is_sub_habitat = TRUE) H
            GROUP BY
                (HABITAT)
	 )a where geno_habitat_datas.habitat_mint = a.habitat;

	END IF;
END $$;


CREATE OR REPLACE FUNCTION UPDATE_DAILY_KI_HARVESTING_CAP ()
    RETURNS TRIGGER
    LANGUAGE PLPGSQL
    AS $EOF$
BEGIN
    IF pg_trigger_depth() <> 1 THEN
        RETURN NEW;
    END IF;
    IF NEW.is_sub_habitat = FALSE THEN
        UPDATE
            geno_habitat_datas
        SET
            daily_ki_harvesting_cap = n.daily_ki_harvesting_cap
        FROM (
            SELECT
                CASE WHEN COUNT(HABITAT) > 1 THEN
                    SUM(HABITAT_CAP) * 1.1
                ELSE
                    SUM(HABITAT_CAP)
                END AS daily_ki_harvesting_cap
            FROM (
                SELECT
                    HABITAT_MINT AS HABITAT,
                    CASE WHEN LEVEL = 1 THEN
                        10000000000 * GENESIS::int + 30000000000
                    WHEN LEVEL = 2 THEN
                        10000000000 * GENESIS::int + 50000000000
                    WHEN LEVEL = 3 THEN
                        10000000000 * GENESIS::int + 70000000000
                    ELSE
                        0
                    END AS HABITAT_CAP
                FROM
                    GENO_HABITAT_DATAS
                WHERE
                    habitat_mint = NEW.habitat_mint
                    AND is_sub_habitat = FALSE
                UNION ALL
                SELECT
                    PARENT_HABITAT AS HABITAT,
                    CASE WHEN LEVEL = 1 THEN
                        10000000000 * GENESIS::int + 30000000000
                    WHEN LEVEL = 2 THEN
                        10000000000 * GENESIS::int + 50000000000
                    WHEN LEVEL = 3 THEN
                        10000000000 * GENESIS::int + 70000000000
                    ELSE
                        0
                    END AS HABITAT_CAP
                FROM
                    GENO_HABITAT_DATAS
                WHERE
                    PARENT_HABITAT = NEW.habitat_mint
                    AND is_sub_habitat = TRUE) H
            GROUP BY
                (HABITAT)) n
    WHERE
        habitat_mint = NEW.habitat_mint;
    ELSE
        UPDATE
            geno_habitat_datas
        SET
            daily_ki_harvesting_cap = n.daily_ki_harvesting_cap
        FROM (
            SELECT
                CASE WHEN COUNT(HABITAT) > 1 THEN
                    SUM(HABITAT_CAP) * 1.1
                ELSE
                    SUM(HABITAT_CAP)
                END AS daily_ki_harvesting_cap
            FROM (
                SELECT
                    HABITAT_MINT AS HABITAT,
                    CASE WHEN LEVEL = 1 THEN
                        10000000000 * GENESIS::int + 30000000000
                    WHEN LEVEL = 2 THEN
                        10000000000 * GENESIS::int + 50000000000
                    WHEN LEVEL = 3 THEN
                        10000000000 * GENESIS::int + 70000000000
                    ELSE
                        0
                    END AS HABITAT_CAP
                FROM
                    GENO_HABITAT_DATAS
                WHERE
                    habitat_mint = NEW.parent_habitat
                    AND is_sub_habitat = FALSE
                UNION ALL
                SELECT
                    PARENT_HABITAT AS HABITAT,
                    CASE WHEN LEVEL = 1 THEN
                        10000000000 * GENESIS::int + 30000000000
                    WHEN LEVEL = 2 THEN
                        10000000000 * GENESIS::int + 50000000000
                    WHEN LEVEL = 3 THEN
                        10000000000 * GENESIS::int + 70000000000
                    ELSE
                        0
                    END AS HABITAT_CAP
                FROM
                    GENO_HABITAT_DATAS
                WHERE
                    PARENT_HABITAT = NEW.parent_habitat
                    AND is_sub_habitat = TRUE) H
            GROUP BY
                (HABITAT)) n
    WHERE
        habitat_mint = NEW.parent_habitat;
    END IF;
	RETURN NULL;
END
$EOF$;

CREATE OR REPLACE TRIGGER UPDATE_DAILY_KI_HARVESTING_CAP_TRIGGER AFTER 
     INSERT ON GENO_HABITAT_DATAS FOR ROW EXECUTE FUNCTION UPDATE_DAILY_KI_HARVESTING_CAP ();

