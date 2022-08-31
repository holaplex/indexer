-- clear all the data
delete from candy_machine_config_lines;

alter table candy_machine_config_lines
rename column candy_machine_address to address;

alter table candy_machine_config_lines
drop constraint candy_machine_address_idx_pkey,
drop column idx,
drop column taken,
add constraint candy_machine_config_lines_pkey primary key (address);