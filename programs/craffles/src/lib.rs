pub mod instructions;
pub mod errors;

use anchor_lang::prelude::*;
//use mpl_bubblegum::state::metaplex_adapter::MetadataArgs;
use instructions::*;

declare_id!("cRafa563zLryDbGePsXaKEHkcY7nSScjszMxh1UjiTJ");

#[program]
pub mod craffles {
    use super::*;

    pub fn create_raffle(ctx: Context<CreateRaffle>, end_timestamp: i64, ticket_price: u64, depth_size_pair: ValidDepthSizePair) -> Result<()> {
        instructions::create_raffle(ctx, end_timestamp, ticket_price, depth_size_pair)
    }

    pub fn buy_tickets(ctx: Context<BuyTickets>, amount: u16, metadata: Metadata) -> Result<()> {
        instructions::buy_tickets(ctx, amount, metadata)
    }
}
