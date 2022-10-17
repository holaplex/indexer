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
    insert into wallet_total_rewards
    (total_reward, wallet_address, reward_center_address)
    values
    (new.buyer_reward, new.buyer, new.reward_center),
    (new.seller_reward, new.seller, new.reward_center)
    on conflict do update
    set total_reward = total_reward + excluded.total_reward
    where reward_center_address = excluded.reward_center_address
    and wallet_address = excluded.wallet_address;
    return null;
end
$$;

create trigger new_rewards_payout
after insert on reward_payouts for row
execute function update_total_rewards();