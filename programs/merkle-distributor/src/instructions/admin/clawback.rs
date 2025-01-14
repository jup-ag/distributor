// Instruction to clawback funds once they have expired

use anchor_lang::{context::Context, prelude::*, Accounts, Key, Result};
use anchor_spl::{
    token,
    token::{Token, TokenAccount},
};

use crate::{error::ErrorCode, state::merkle_distributor::MerkleDistributor};

/// [merkle_distributor::clawback] accounts.
#[derive(Accounts)]
pub struct Clawback<'info> {
    /// The [MerkleDistributor].
    #[account(mut, has_one = clawback_receiver)]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// Distributor ATA containing the tokens to distribute.
    #[account(
        mut,
        associated_token::mint = distributor.load()?.mint,
        associated_token::authority = distributor.key(),
        address = distributor.load()?.token_vault
    )]
    pub from: Account<'info, TokenAccount>,

    /// The Clawback token account.
    #[account(mut)]
    pub clawback_receiver: Account<'info, TokenAccount>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Claws back unclaimed tokens by:
/// 1. Checking that the lockup has expired
/// 2. Transferring remaining funds from the vault to the clawback receiver
/// 3. Marking the distributor as clawed back
/// CHECK:
///     1. The distributor has not already been clawed back
#[allow(clippy::result_large_err)]
pub fn handle_clawback(ctx: Context<Clawback>) -> Result<()> {
    let mut distributor = ctx.accounts.distributor.load_mut()?;

    require!(
        !distributor.clawed_back(),
        ErrorCode::ClawbackAlreadyClaimed
    );

    let curr_ts = Clock::get()?.unix_timestamp;

    if curr_ts < distributor.clawback_start_ts {
        return Err(ErrorCode::ClawbackBeforeStart.into());
    }

    distributor.set_clawed_back();

    let signer = distributor.signer();
    drop(distributor);
    let seeds = signer.seeds();

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.clawback_receiver.to_account_info(),
                authority: ctx.accounts.distributor.to_account_info(),
            },
        )
        .with_signer(&[&seeds[..]]),
        ctx.accounts.from.amount,
    )?;

    Ok(())
}
