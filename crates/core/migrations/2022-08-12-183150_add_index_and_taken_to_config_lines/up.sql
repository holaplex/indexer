-- Your SQL goes here
alter table candy_machine_config_lines
add column idx   int not null,
add column taken bool not null;

alter table candy_machine_config_lines
rename column address to candy_machine_address;

alter table candy_machine_config_lines
drop constraint candy_machine_config_lines_pkey;

alter table candy_machine_config_lines
add constraint candy_machine_address_idx_pkey primary key (candy_machine_address, idx);