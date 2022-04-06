use objects::stats::MintStats;

use super::{charts::MintCharts, prelude::*};

#[derive(Debug, Clone)]
/// A Metaplex auction house
pub struct AuctionHouse {
    pub address: String,
    /// Mint address of the token in which fees are vendored
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
    /// Account for which fees are paid out to
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

#[graphql_object(Context = AppContext)]
impl AuctionHouse {
    pub async fn stats(&self, context: &AppContext) -> FieldResult<Option<MintStats>> {
        context
            .mint_stats_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(arguments(
        start_date(description = "Start date for which we want to get the average price"),
        end_date(description = "End date for which we want to get the average price")
    ))]
    pub async fn charts(&self, context: &AppContext) -> FieldResult<Option<MintCharts>> {
        MintCharts {
            auction_house: self.address.clone(),
            start_date,
            end_date,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn treasury_mint(&self) -> &str {
        &self.treasury_mint
    }

    pub fn auction_house_treasury(&self) -> &str {
        &self.auction_house_treasury
    }

    pub fn treasury_withdrawal_destination(&self) -> &str {
        &self.treasury_withdrawal_destination
    }

    pub fn fee_withdrawal_destination(&self) -> &str {
        &self.fee_withdrawal_destination
    }

    pub fn authority(&self) -> &str {
        &self.authority
    }

    pub fn creator(&self) -> &str {
        &self.creator
    }

    pub fn bump(&self) -> i32 {
        self.bump
    }

    pub fn treasury_bump(&self) -> i32 {
        self.treasury_bump
    }

    pub fn fee_payer_bump(&self) -> i32 {
        self.fee_payer_bump
    }

    pub fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    pub fn requires_sign_off(&self) -> bool {
        self.requires_sign_off
    }

    pub fn can_change_sale_price(&self) -> bool {
        self.can_change_sale_price
    }

    pub fn auction_house_fee_account(&self) -> &str {
        &self.auction_house_fee_account
    }
}
