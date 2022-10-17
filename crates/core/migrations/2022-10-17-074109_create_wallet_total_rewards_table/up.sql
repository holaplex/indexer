create table wallet_total_rewards (
    id uuid primary key default gen_random_uuid(),
    wallet_address varchar(48) not null,
    reward_center_address varchar(48) not null,
    total_reward numeric not null
);

create index wallet_total_rewards_wallet_address_idx
on wallet_total_rewards (wallet_address);

create index wallet_total_rewards_reward_center_address_idx
on wallet_total_rewards (reward_center_address);

create function update_total_rewards() returns trigger
    language plpgsql
    as $$
begin
    update wallet_total_rewards set total_reward =
    case 
        when wallet_address in
        (
            select wallet_address
            from reward_payouts
            where wallet_address = new.buyer
        )
        then total_reward + new.buyer_reward
        when wallet_address in
        (
            select wallet_address
            from reward_payouts
            where wallet_address = new.seller
        )
        then total_reward + new.seller_reward
    end
    where reward_center_address = new.reward_center;
    return null;
end
$$;

create trigger new_rewards_payout
after insert on reward_payouts for row
execute function update_total_rewards();