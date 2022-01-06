delete from editions;
delete from master_editions;

alter table editions
add column metadata_address varchar(48) unique not null,
add foreign key (metadata_address) references metadatas (address);

alter table master_editions
add column metadata_address varchar(48) unique not null,
add foreign key (metadata_address) references metadatas (address);
