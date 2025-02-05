use anchor_lang::{
    accounts::{account::Account, program::Program, signer::Signer},
    context::{Context, CpiContext},
    emit,
    prelude::*,
    Accounts, Result, ToAccountInfo,
};
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    error::ErrorCode,
    state::{
        claim_status::ClaimStatus, claimed_event::ClaimedEvent,
        merkle_distributor::MerkleDistributor,
    },
};
use locked_voter::{self as voter, Escrow};
use locked_voter::{program::LockedVoter as Voter, Locker};

/// [merkle_distributor::claim_locked] accounts.
#[derive(Accounts)]
pub struct ClaimLockedAndStake<'info> {
    /// The [MerkleDistributor].
    #[account(mut, has_one = locker)]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// Claim Status PDA
    #[account(
        mut,
        has_one = distributor,
        has_one = claimant,
    )]
    pub claim_status: AccountLoader<'info, ClaimStatus>,

    /// Distributor ATA containing the tokens to distribute.
    #[account(
        mut,
        associated_token::mint = distributor.load()?.mint,
        associated_token::authority = distributor.key(),
        address = distributor.load()?.token_vault,
    )]
    pub from: Account<'info, TokenAccount>,

    /// Who is claiming the tokens.
    pub claimant: Signer<'info>,

    /// operator
    pub operator: Option<Signer<'info>>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,

    /// Voter program
    pub voter_program: Program<'info, Voter>,

    /// CHECK: Locker
    #[account(mut)]
    pub locker: Box<Account<'info, Locker>>,

    /// CHECK: escrow
    #[account(mut)]
    pub escrow: Box<Account<'info, Escrow>>,

    /// CHECK: escrow_tokens
    #[account(mut)]
    pub escrow_tokens: UncheckedAccount<'info>,
}

/// Claim locked tokens as they become unlocked.
/// Check:
///     1. The claim window has not expired and the distributor has not been clawed back
///     2. The withdraw-able amount is greater than 0
///     3. The locked amount withdrawn is ≤ than the locked amount
///     4. The distributor amount claimed is ≤ than the max total claim
#[allow(clippy::result_large_err)]
pub fn handle_claim_locked_and_stake(ctx: Context<ClaimLockedAndStake>) -> Result<()> {
    let mut distributor = ctx.accounts.distributor.load_mut()?;

    require!(!distributor.clawed_back(), ErrorCode::ClaimExpired);

    // check operator
    distributor.authorize_claim_and_stake(&ctx.accounts.operator)?;

    let mut claim_status = ctx.accounts.claim_status.load_mut()?;

    let curr_ts = Clock::get()?.unix_timestamp;

    let escrow = &ctx.accounts.escrow;
    require!(escrow.is_max_lock, ErrorCode::EscrowIsNotMaxLock);

    require!(!distributor.clawed_back(), ErrorCode::ClaimExpired);

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

    let signer = distributor.signer();
    drop(distributor);
    let seeds = signer.seeds();

    let seeds = &[&seeds[..]];

    // CPI to voter
    let cpi_ctx = CpiContext::new(
        ctx.accounts.voter_program.to_account_info(),
        voter::cpi::accounts::IncreaseLockedAmount {
            locker: ctx.accounts.locker.to_account_info(),
            escrow: ctx.accounts.escrow.to_account_info(),
            escrow_tokens: ctx.accounts.escrow_tokens.to_account_info(),
            payer: ctx.accounts.distributor.to_account_info(),
            source_tokens: ctx.accounts.from.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    )
    .with_signer(seeds);

    voter::cpi::increase_locked_amount(cpi_ctx, amount)?;

    emit!(ClaimedEvent {
        claimant: ctx.accounts.claimant.key(),
        amount,
    });
    Ok(())
}
