alter table candy_machine_config_lines
rename column address to candy_machine_address;

alter table candy_machine_config_lines
add column idx int not null,
add column taken bool not null,
drop constraint candy_machine_config_lines_pkey,
add constraint candy_machine_address_idx_pkey primary key (candy_machine_address, idx);