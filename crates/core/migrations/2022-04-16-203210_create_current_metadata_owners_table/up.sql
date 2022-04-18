create table if not exists current_metadata_owners as
select distinct on (mint_address) mint_address,
                   owner_address,
                   address as token_account_address,
                   updated_at,
                   slot
from token_accounts
where amount = 1 and slot is not null
order by mint_address, slot desc;

create index if not exists metadata_owners_owner_addr_index on current_metadata_owners 
using hash (owner_address);

alter table current_metadata_owners
alter updated_at
set default now();

alter table current_metadata_owners
alter updated_at
set not null;

alter table current_metadata_owners
alter owner_address
set not null;

alter table current_metadata_owners
alter token_account_address
set not null;

alter table current_metadata_owners
alter slot
set not null;

do $$
begin if not exists
  (select 1
   from pg_trigger
   where tgname = 'set_curr_metadata_owner_updated_at_column') then
        create trigger set_curr_metadata_owner_updated_at_column
        before
        update on current_metadata_owners
        for each row execute procedure trigger_set_updated_at_timestamp(); 
    end if;
end
$$;

do $$
begin if not exists
  (select constraint_name
   from information_schema.constraint_column_usage
   where constraint_name = 'metadata_current_owners_primary_key') then 
        execute 'alter table current_metadata_owners
            add constraint metadata_current_owners_primary_key 
            primary key (mint_address)';
end if;
end
$$;