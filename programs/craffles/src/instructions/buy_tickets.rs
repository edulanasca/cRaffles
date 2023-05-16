use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use mpl_bubblegum::state::{TreeConfig, metaplex_adapter::MetadataArgs};
use mpl_bubblegum::state::metaplex_adapter::{TokenStandard, Collection, Creator, Uses, TokenProgramVersion};
use spl_account_compression::{Noop, program::SplAccountCompression};
use crate::instructions::{Raffle, Bubblegum};
use crate::errors::RaffleError;

/*#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TokenStandard {
    NonFungible,        // This is a master edition
    FungibleAsset,      // A token with metadata that can also have attrributes
    Fungible,           // A token with simple metadata
    NonFungibleEdition, // This is a limited edition
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Uses {
    // 17 bytes + Option byte
    pub use_method: UseMethod, //1
    pub remaining: u64,        //8
    pub total: u64,            //8
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum TokenProgramVersion {
    Original,
    Token2022,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}*/

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Metadata {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    // Immutable, once flipped, all sales of this metadata are considered secondary.
    pub primary_sale_happened: bool,
    // Whether or not the data struct is mutable, default is not
    pub is_mutable: bool,
    /// nonce for easy calculation of editions, if present
    pub edition_nonce: Option<u8>,
    /// Since we cannot easily change Metadata, we add the new DataV2 fields here at the end.
    pub token_standard: Option<TokenStandard>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
    pub token_program_version: TokenProgramVersion,
    pub creators: Vec<Creator>,
}

pub fn buy_tickets(ctx: Context<BuyTickets>, amount: u16, metadata: Metadata) -> Result<()> {
    let clock = Clock::get()?;
    let raffle = &mut ctx.accounts.raffle;

    if clock.unix_timestamp > raffle.end_timestamp {
        return err!(RaffleError::RaffleEnded);
    }

    mpl_bubblegum::cpi::mint_v1(
        CpiContext::new(
            ctx.accounts.bubblegum_program.to_account_info(),
            mpl_bubblegum::cpi::accounts::MintV1 {
                tree_authority: ctx.accounts.tree_authority.to_account_info(),
                leaf_owner: ctx.accounts.leaf_owner.to_account_info(),
                leaf_delegate: ctx.accounts.leaf_delegate.to_account_info(),
                merkle_tree: ctx.accounts.merkle_tree.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                tree_delegate: ctx.accounts.buyer.to_account_info(),
                log_wrapper: ctx.accounts.log_wrapper.to_account_info(),
                compression_program: ctx.accounts.compression_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            }
        ),
        MetadataArgs {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            seller_fee_basis_points: metadata.seller_fee_basis_points,
            primary_sale_happened: metadata.primary_sale_happened,
            is_mutable: metadata.is_mutable,
            edition_nonce: metadata.edition_nonce,
            token_standard: metadata.token_standard,
            collection: metadata.collection,
            uses: metadata.uses,
            token_program_version: metadata.token_program_version,
            creators: metadata.creators,
        }
    )?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_token_account.to_account_info(),
                to: ctx.accounts.proceeds.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        raffle.ticket_price
            .checked_mul(amount as u64)
            .ok_or(RaffleError::InvalidCalculation)?,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct BuyTickets<'info> {
    pub raffle: Account<'info, Raffle>,
    #[account(
    mut,
    seeds = [b"proceeds", raffle.key().as_ref()],
    bump,
    )]
    pub proceeds: Account<'info, TokenAccount>,
    #[account(
    seeds = [merkle_tree.key().as_ref()],
    bump,
    seeds::program = bubblegum_program.key()
    )]
    pub tree_authority: Account<'info, TreeConfig>,
    /// CHECK: This account is neither written to nor read from.
    pub leaf_owner: AccountInfo<'info>,
    /// CHECK: This account is neither written to nor read from.
    pub leaf_delegate: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: unsafe
    pub merkle_tree: UncheckedAccount<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub bubblegum_program: Program<'info, Bubblegum>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}