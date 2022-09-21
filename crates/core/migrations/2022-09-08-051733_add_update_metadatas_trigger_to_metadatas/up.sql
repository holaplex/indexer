create or replace function update_metadatas() returns trigger
  language plpgsql
  as $EOF$
begin
  if (old.slot > new.slot and old.burned_at is null and new.burned_at is null) OR old.burned_at is not null then
    return old;
  end if;

  return new;
end
$EOF$;


create trigger update_metadatas_trigger
before update on metadatas for row
execute function update_metadatas();