drop trigger graph_connections_check_slot_wv on graph_connections;

alter table graph_connections
drop column slot,
drop column write_version;