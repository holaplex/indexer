create table reward_centers (
  address varchar(48) primary key,
  token_mint varchar(48) not null,
  auction_house varchar(48) not null,
  bump smallint not null,
  slot bigint not null default -1,
  write_version bigint not null
);

create type payout_operation 
as enum ('Multiple', 'Divide');

create table reward_rules (
  reward_center_address varchar(48) primary key,
  seller_reward_payout_basis_points smallint not null,
  mathematical_operand payout_operation not null,
  payout_numeral smallint not null,
  foreign key (reward_center_address) references reward_centers (address)
);

create trigger rewards_centers_check_slot_wv
before update on reward_centers for row
execute function check_slot_wv();