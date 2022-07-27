alter table geno_habitat_datas
rename column harvester to harvester_text;

alter table geno_habitat_datas
add column if not exists harvester_bytes bytea not null default ''::bytea;

alter table geno_habitat_datas
alter column harvester_bytes drop default;

alter table geno_habitat_datas
rename column harvester_bytes to harvester;
