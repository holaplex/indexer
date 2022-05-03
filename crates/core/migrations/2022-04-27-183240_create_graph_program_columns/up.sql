alter table graph_connections
add column connected_at timestamp;

alter table graph_connections
add column disconnected_at timestamp;

update graph_connections
set connected_at = CURRENT_TIMESTAMP();

alter table graph_connections
alter column connected_at set not null;