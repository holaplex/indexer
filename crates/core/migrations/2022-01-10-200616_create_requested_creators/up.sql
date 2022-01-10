create table creators (
  address bytea primary key not null,
  created_at              timestamp not null default now(),
  updated_at              timestamp null
);
