-- This file should undo anything in `up.sql`
alter table candy_machine_creators
drop constraint candy_machine_creators_compound_pkey,
add constraint candy_machine_creators_pkey primary key (candy_machine_creators_compound_pkey);