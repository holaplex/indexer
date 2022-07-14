alter table metadatas
add column burned bool not null default false;

create index if not exists metadatas_burned_idx on 
  metadatas (burned);
