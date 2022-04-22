
alter table twitter_handle_name_services
add column from_bonfida boolean not null default true;

alter table twitter_handle_name_services
add column from_cardinal boolean not null default false;

alter table twitter_handle_name_services
add column write_version bigint not null default 0;
