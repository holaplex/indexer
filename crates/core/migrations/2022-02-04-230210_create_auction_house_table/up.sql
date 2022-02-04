create table auction_houses (
  address                         varchar(48) primary key,
  treasury_mint                   varchar(48) not null,
  auction_house_treasury          varchar(48) not null,
  treasury_withdrawal_destination varchar(48) not null,  
  fee_withdrawal_destination      varchar(48) not null,
  authority                       varchar(48) not null,   
  creator                         varchar(48) not null,  
  bump                            smallint    not null,
  treasury_bump                   smallint    not null, 
  fee_payer_bump                  smallint    not null,
  seller_fee_basis_points         smallint    not null,
  requires_sign_off               boolean     not null,
  can_change_sale_price           boolean     not null
);