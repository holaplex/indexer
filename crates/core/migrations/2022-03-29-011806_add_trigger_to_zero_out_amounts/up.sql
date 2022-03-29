create function set_amount_zero_for_last_token_acc()
returns trigger
as $$
begin
	if 
    (select amount from token_accounts 
    where mint_address = new.mint_address order by updated_at desc limit 1) = 1 
    and new.amount = 1
	then
		update token_accounts set amount = 0 where mint_address = new.mint_address 
        and updated_at in 
        (select updated_at 
        from token_accounts 
        where mint_address = new.mint_address order by updated_at desc limit 1 );
	end if;
	return new;
end;
$$
language plpgsql;

create  trigger zero_out_last_token_account_if_one before insert
on token_accounts
for each row
execute procedure set_amount_zero_for_last_token_acc();
