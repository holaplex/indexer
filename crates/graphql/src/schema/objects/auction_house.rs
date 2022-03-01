use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct AuctionHouse {
    pub address: String,
    pub treasury_mint: String,
    pub auction_house_treasury: String,
    pub treasury_withdrawal_destination: String,
    pub fee_withdrawal_destination: String,
    pub authority: String,
    pub creator: String,
    pub bump: i32,
    pub treasury_bump: i32,
    pub fee_payer_bump: i32,
    pub seller_fee_basis_points: i32,
    pub requires_sign_off: bool,
    pub can_change_sale_price: bool,
    pub auction_house_fee_account: String,
}

impl<'a> From<models::AuctionHouse<'a>> for AuctionHouse {
    fn from(
        models::AuctionHouse {
            address,
            treasury_mint,
            auction_house_treasury,
            treasury_withdrawal_destination,
            fee_withdrawal_destination,
            authority,
            creator,
            bump,
            treasury_bump,
            fee_payer_bump,
            seller_fee_basis_points,
            requires_sign_off,
            can_change_sale_price,
            auction_house_fee_account,
        }: models::AuctionHouse,
    ) -> Self {
        Self {
            address: address.into_owned(),
            treasury_mint: treasury_mint.into_owned(),
            auction_house_treasury: auction_house_treasury.into_owned(),
            treasury_withdrawal_destination: treasury_withdrawal_destination.into_owned(),
            fee_withdrawal_destination: fee_withdrawal_destination.into_owned(),
            authority: authority.into_owned(),
            creator: creator.into_owned(),
            bump: bump.into(),
            treasury_bump: treasury_bump.into(),
            fee_payer_bump: fee_payer_bump.into(),
            seller_fee_basis_points: seller_fee_basis_points.into(),
            requires_sign_off,
            can_change_sale_price,
            auction_house_fee_account: auction_house_fee_account.into_owned(),
        }
    }
}
