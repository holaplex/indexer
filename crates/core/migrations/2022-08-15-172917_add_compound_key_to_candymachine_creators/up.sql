-- Your SQL goes here
alter table candy_machine_creators
drop constraint candy_machine_creators_pkey,
add constraint candy_machine_creators_compound_pkey primary key (candy_machine_address, creator_address);