use super::prelude::*;

#[derive(Debug, Clone)]
pub struct AuctionHouse {
    pub address: String,
    pub authority: String,
    pub seller_fee_basis_points: i32,
    pub auction_house_fee_account: String,
}

impl<'a> From<models::AuctionHouse<'a>> for AuctionHouse {
    fn from(
        models::AuctionHouse {
            address,
            treasury_mint: _,
            auction_house_treasury: _,
            treasury_withdrawal_destination: _,
            fee_withdrawal_destination: _,
            authority,
            creator: _,
            bump: _,
            treasury_bump: _,
            fee_payer_bump: _,
            seller_fee_basis_points,
            requires_sign_off: _,
            can_change_sale_price: _,
            auction_house_fee_account,
            ..
        }: models::AuctionHouse,
    ) -> Self {
        Self {
            address: address.into_owned(),
            authority: authority.into_owned(),
            seller_fee_basis_points: seller_fee_basis_points.into(),
            auction_house_fee_account: auction_house_fee_account.into_owned(),
        }
    }
}

#[graphql_object(Context = AppContext)]
impl AuctionHouse {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn authority(&self) -> String {
        self.authority.clone()
    }

    pub fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    pub fn auction_house_fee_account(&self) -> String {
        self.auction_house_fee_account.clone()
    }
}
