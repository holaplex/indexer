create table sol_domains (
    address                      varchar(48)    primary key,
    owner                        varchar(48)    not null,
    name                         text           not null,
    slot                         bigint         not null                      
);

create index if not exists sol_domains_owner_idx
on sol_domains (owner);

create index if not exists sol_domains_name_idx
on sol_domains (name);