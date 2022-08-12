alter table metadatas drop column burned;
alter table metadatas add column burned_at timestamp null;