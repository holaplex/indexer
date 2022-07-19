alter table graph_connections
add column slot          bigint not null default 0,
add column write_version bigint not null default 0;

create trigger graph_connections_check_slot_wv
before update on graph_connections for row
execute function check_slot_wv();