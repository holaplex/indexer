create function zero_out_all_token_accounts()
returns trigger
as $$
begin
	if (select sum(amount) from token_accounts where mint_address = new.mint_address) > 0 
    and new.amount = 1
	then
		update token_accounts set amount = 0 where mint_address = new.mint_address;
	end if;
	return new;
end;
$$
language plpgsql;

create  trigger set_amount_to_zero before insert
on token_accounts
for each row
execute procedure zero_out_all_token_accounts();