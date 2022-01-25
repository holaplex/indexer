create table metadata_jsons (
  metadata_address  varchar(48) primary key not null,
  fingerprint       bytea,
  description       text,
  image             text,
  animation_url     text,
  external_url      text,
  category          text,
  updated_at        timestamp,
  raw_content       jsonb
);

create table attributes (
  metadata_address  varchar(48) not null,
  name              text,
  value             text,
  trait_type        text,
  id                uuid primary key default gen_random_uuid()
);

create table files (
  metadata_address  varchar(48) not null,
  uri               text,
  file_type         text,
  id                uuid primary key default gen_random_uuid()
);

create table metadata_collections (
  metadata_address  varchar(48) not null,
  name              text,
  family            text,
  id                uuid primary key default gen_random_uuid()
);

create index metadata_jsons_metadata_address_index
on metadata_jsons (metadata_address);