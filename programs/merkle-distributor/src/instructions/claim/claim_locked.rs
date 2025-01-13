use anchor_lang::{
    accounts::{account::Account, program::Program, signer::Signer},
    context::{Context, CpiContext},
    emit,
    prelude::*,
    Accounts, Result, ToAccountInfo,
};
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{
    error::ErrorCode,
    state::{
        claim_status::ClaimStatus, claimed_event::ClaimedEvent,
        merkle_distributor::MerkleDistributor,
    },
};

/// [merkle_distributor::claim_locked] accounts.
#[derive(Accounts)]
pub struct ClaimLocked<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// Claim Status PDA
    #[account(
        mut,
        has_one = distributor,
        has_one = claimant,
    )]
    pub claim_status: Account<'info, ClaimStatus>,

    /// Distributor ATA containing the tokens to distribute.
    #[account(
        mut,
        associated_token::mint = distributor.load()?.mint,
        associated_token::authority = distributor.key(),
        address = distributor.load()?.token_vault,
    )]
    pub from: Account<'info, TokenAccount>,

    /// Account to send the claimed tokens to.
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    /// Who is claiming the tokens.
    pub claimant: Signer<'info>,

    /// operator
    pub operator: Option<Signer<'info>>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Claim locked tokens as they become unlocked.
/// Check:
///     1. The claim window has not expired and the distributor has not been clawed back
///     2. The withdraw-able amount is greater than 0
///     3. The locked amount withdrawn is ≤ than the locked amount
///     4. The distributor amount claimed is ≤ than the max total claim
#[allow(clippy::result_large_err)]
pub fn handle_claim_locked(ctx: Context<ClaimLocked>) -> Result<()> {
    let mut distributor = ctx.accounts.distributor.load_mut()?;

    let claim_status = &mut ctx.accounts.claim_status;
    let curr_ts = Clock::get()?.unix_timestamp;

    require!(!distributor.clawed_back(), ErrorCode::ClaimExpired);

    // check operator
    distributor.validate_claim(&ctx.accounts.operator)?;

    let activation_handler = distributor.get_activation_handler()?;
    activation_handler.validate_claim()?;

    let amount =
        claim_status.amount_withdrawable(curr_ts, distributor.start_ts, distributor.end_ts)?;

    require!(amount > 0, ErrorCode::InsufficientUnlockedTokens);

    claim_status.locked_amount_withdrawn = claim_status
        .locked_amount_withdrawn
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticError)?;

    require!(
        claim_status.locked_amount_withdrawn <= claim_status.locked_amount,
        ErrorCode::ExceededMaxClaim
    );

    distributor.total_amount_claimed = distributor
        .total_amount_claimed
        .checked_add(amount)
        .ok_or(ErrorCode::ArithmeticError)?;

    require!(
        distributor.total_amount_claimed <= distributor.max_total_claim,
        ErrorCode::ExceededMaxClaim
    );

    let remaining_seconds = match curr_ts < distributor.end_ts {
        true => distributor.end_ts - curr_ts,
        false => 0,
    };

    let days = remaining_seconds / (24 * 60 * 60); // number of days
    let seconds_after_days = remaining_seconds % (24 * 60 * 60); // Remaining seconds after subtracting full days

    // Note: might get truncated, do not rely on
    msg!(
        "Withdrew amount {} with {} days and {} seconds left in lockup",
        amount,
        days,
        seconds_after_days,
    );

    let base = distributor.base;
    let mint = distributor.mint;
    let version = distributor.version;
    let bump = distributor.bump;
    drop(distributor);

    let seeds = [
        b"MerkleDistributor".as_ref(),
        &base.to_bytes(),
        &mint.to_bytes(),
        &version.to_le_bytes(),
        &[bump],
    ];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.distributor.to_account_info(),
            },
        )
        .with_signer(&[&seeds[..]]),
        amount,
    )?;

    emit!(ClaimedEvent {
        claimant: ctx.accounts.claimant.key(),
        amount,
    });
    Ok(())
}
