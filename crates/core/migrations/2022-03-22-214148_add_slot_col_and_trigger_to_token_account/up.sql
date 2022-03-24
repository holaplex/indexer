alter table token_accounts
add column slot bigint null;

alter table token_accounts alter updated_at set default now();

create  function trigger_set_updated_at_timestamp()
returns trigger as $$
begin
    new.updated_at = now();
    return new;
end;
$$ language 'plpgsql';

create trigger set_token_account_updated_at
before update on token_accounts
for each row
execute procedure trigger_set_updated_at_timestamp();

