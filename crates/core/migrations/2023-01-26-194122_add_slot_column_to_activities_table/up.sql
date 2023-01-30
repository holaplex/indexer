alter table marketplace_activities
add column slot bigint not null default -1;

create index if not exists marketplace_activities_slot_idx on marketplace_activities(slot);