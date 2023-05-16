use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::mem::size_of;
use spl_account_compression::Noop;
use spl_account_compression::program::SplAccountCompression;

#[derive(Clone)]
pub struct Bubblegum {}

impl Id for Bubblegum {
    fn id() -> Pubkey {
        mpl_bubblegum::id()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ValidDepthSizePair {
    max_depth: u32,
    max_buffer_size: u32
}

pub fn create_raffle(
    ctx: Context<CreateRaffle>,
    end_timestamp: i64,
    ticket_price: u64,
    depth_size_pair: ValidDepthSizePair
) -> Result<()> {
    let raffle = &mut ctx.accounts.raffle;
    raffle.creator = ctx.accounts.creator.key();
    raffle.merkle_tree = ctx.accounts.merkle_tree.key();
    raffle.end_timestamp = end_timestamp;
    raffle.ticket_price = ticket_price;

    mpl_bubblegum::cpi::create_tree(
        CpiContext::new(
            ctx.accounts.bubblegum_program.to_account_info(),
            mpl_bubblegum::cpi::accounts::CreateTree {
                tree_authority: ctx.accounts.tree_authority.to_account_info(),
                merkle_tree: ctx.accounts.merkle_tree.to_account_info(),
                payer: ctx.accounts.creator.to_account_info(),
                tree_creator: ctx.accounts.creator.to_account_info(),
                log_wrapper: ctx.accounts.log_wrapper.to_account_info(),
                compression_program: ctx.accounts.compression_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            }
        ),
        depth_size_pair.max_depth, depth_size_pair.max_buffer_size, Some(false)
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateRaffle<'info> {
    #[account(
    init,
    payer = creator,
    space = 8 + size_of::< Raffle > (),
    seeds = [b"raffle".as_ref(), merkle_tree.key().as_ref()],
    bump,
    )]
    pub raffle: Account<'info, Raffle>,
    #[account(
    init,
    seeds = [b"proceeds", raffle.key().as_ref()],
    bump,
    payer = creator,
    token::mint = proceeds_mint,
    token::authority = raffle,
    )]
    pub proceeds: Account<'info, TokenAccount>,
    pub proceeds_mint: Account<'info, Mint>,
    #[account(zero)]
    /// CHECK: This account must be all zeros
    pub merkle_tree: UncheckedAccount<'info>, // replaces entrants
    /// CHECK: Will be initialized in the CPI
    #[account(mut)]
    pub tree_authority: AccountInfo<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub bubblegum_program: Program<'info, Bubblegum>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct Raffle {
    pub creator: Pubkey,
    pub end_timestamp: i64,
    pub ticket_price: u64,
    pub merkle_tree: Pubkey,
}