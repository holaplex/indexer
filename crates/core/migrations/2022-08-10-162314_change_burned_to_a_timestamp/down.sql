alter table metadatas drop column burned_at;
alter table metadatas add column burned timestamp null;