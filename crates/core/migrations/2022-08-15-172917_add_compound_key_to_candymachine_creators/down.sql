-- drop duplicates from candy_machine creators
delete from candy_machine_creators a using candy_machine_creators b
where a.candy_machine_address=b.candy_machine_address and a.creator_address > b.creator_address;

alter table candy_machine_creators
drop constraint candy_machine_creators_compound_pkey,
add constraint candy_machine_creators_pkey primary key (candy_machine_address);