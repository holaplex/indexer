ALTER TYPE token_standard ADD VALUE 'ProgrammableNonFungible' AFTER 'NonFungibleEdition';

create type programmable_config 
as enum ('V1');

create table metadata_programmable_configs (
    metadata_address varchar(48) primary key,
    variant programmable_config not null,
    rule_set varchar(48)
);