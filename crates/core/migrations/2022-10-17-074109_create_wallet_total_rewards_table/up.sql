create table wallet_total_rewards (
    wallet_address varchar(48) not null,
    reward_center_address varchar(48) not null,
    total_reward numeric not null,
    primary key (wallet_address, reward_center_address)
);

create index wallet_total_rewards_wallet_address_idx
on wallet_total_rewards (wallet_address);

create index wallet_total_rewards_reward_center_address_idx
on wallet_total_rewards (reward_center_address);

create function update_total_rewards() returns trigger
    language plpgsql
    as $$
begin
    insert into wallet_total_rewards values (new.buyer, new.reward_center, new.buyer_reward)
    on conflict (wallet_address, reward_center_address)
    do update set total_reward = total_reward + new.buyer_reward;

    insert into wallet_total_rewards values (new.seller, new.reward_center, new.seller_reward)
    on conflict (wallet_address, reward_center_address)
    do update set total_reward = total_reward + new.seller_reward;

    return null;
end
$$;

create trigger new_rewards_payout
after insert on reward_payouts for row
execute function update_total_rewards();