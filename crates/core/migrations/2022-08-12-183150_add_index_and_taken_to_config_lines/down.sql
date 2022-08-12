-- This file should undo anything in `up.sql`
alter table candy_machine_config_lines
drop column idx,
drop column taken;

alter table candy_machine_config_lines
rename column candy_machine_address to address;

alter table candy_machine_config_lines
alter table drop constraint candy_machine_address_idx_pkey;

alter table candy_machine_config_lines
add constraint candy_machine_config_lines_pkey primary key (address);