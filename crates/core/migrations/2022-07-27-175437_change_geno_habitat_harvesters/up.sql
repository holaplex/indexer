alter table geno_habitat_datas
rename column harvester to harvester_bytes;

alter table geno_habitat_datas
add column if not exists harvester_text varchar(48) not null default '';

alter table geno_habitat_datas
alter column harvester_text drop default;

alter table geno_habitat_datas
rename column harvester_text to harvester;
