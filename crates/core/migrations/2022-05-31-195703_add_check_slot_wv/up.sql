create function check_slot_wv() returns trigger
  language plpgsql
  as $EOF$
begin
  if (old.slot, old.write_version) > (new.slot, new.write_version) then
    return old;
  end if;

  return new;
end
$EOF$;

create trigger attributes_check_slot_wv
before update on attributes for row
execute function check_slot_wv();

create trigger files_check_slot_wv
before update on files for row
execute function check_slot_wv();

create trigger metadata_collections_check_slot_wv
before update on metadata_collections for row
execute function check_slot_wv();

create trigger metadata_jsons_check_slot_wv
before update on metadata_jsons for row
execute function check_slot_wv();
