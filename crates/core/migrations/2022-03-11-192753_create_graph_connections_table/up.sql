create table graph_connections (
  address           varchar(48) primary key,
  from_account      varchar(48) not null,
  to_account        varchar(48) not null
  );

create index if not exists graph_connections_from_addr_idx
on graph_connections (from_account);

create index if not exists graph_connections_to_addr_idx
on graph_connections (to_account);