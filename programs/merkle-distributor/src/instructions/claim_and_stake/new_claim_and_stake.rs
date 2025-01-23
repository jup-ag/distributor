use crate::state::partial_merkle_tree::PartialMerkleTree;
use crate::LEAF_PREFIX;
use crate::{
    error::ErrorCode,
    state::{
        claim_status::ClaimStatus, claimed_event::NewClaimEvent,
        merkle_distributor::MerkleDistributor,
    },
};
use anchor_lang::{
    context::Context, prelude::*, solana_program::hash::hashv, system_program::System, Accounts,
    Key, Result,
};
use anchor_spl::token::{Token, TokenAccount};
use jito_merkle_verify::verify_partial_merkle;

use locked_voter::program::LockedVoter as Voter;
use locked_voter::{self as voter, Escrow, Locker};

/// [merkle_distributor::new_claim_and_stake] accounts.
#[derive(Accounts)]
pub struct NewClaimAndStake<'info> {
    /// The [MerkleDistributor].
    #[account(mut, has_one = locker)]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    /// The [PartialMerkleTree].
    #[account(has_one = distributor)]
    pub partial_merkle_tree: Account<'info, PartialMerkleTree>,

    /// Claim status PDA
    #[account(
        init,
        seeds = [
            b"ClaimStatus".as_ref(),
            claimant.key().to_bytes().as_ref(),
            distributor.key().to_bytes().as_ref()
        ],
        bump,
        space = 8 + ClaimStatus::INIT_SPACE,
        payer = claimant
    )]
    pub claim_status: AccountLoader<'info, ClaimStatus>,

    /// Distributor ATA containing the tokens to distribute.
    #[account(
        mut,
        associated_token::mint = distributor.load()?.mint,
        associated_token::authority = distributor.key(),
        address = distributor.load()?.token_vault
    )]
    pub from: Account<'info, TokenAccount>,

    /// Who is claiming the tokens.
    #[account(mut)]
    pub claimant: Signer<'info>,

    /// operator
    pub operator: Option<Signer<'info>>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

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

/// Initializes a new claim from the [MerkleDistributor].
/// 1. Increments num_nodes_claimed by 1
/// 2. Initializes claim_status
/// 3. Transfers claim_status.unlocked_amount to the claimant
/// 4. Increments total_amount_claimed by claim_status.unlocked_amount
/// CHECK:
///     1. The claim window has not expired and the distributor has not been clawed back
///     2. The claimant is the owner of the to account
///     3. Num nodes claimed is less than max_num_nodes
///     4. The merkle proof is valid
#[allow(clippy::result_large_err)]
pub fn handle_new_claim_and_stake(
    ctx: Context<NewClaimAndStake>,
    amount_unlocked: u64,
    amount_locked: u64,
    proof: Vec<[u8; 32]>,
    initial_index: u8,
) -> Result<()> {
    let mut distributor = ctx.accounts.distributor.load_mut()?;
    let partial_merkle_tree = &ctx.accounts.partial_merkle_tree;
    require!(!distributor.clawed_back(), ErrorCode::ClaimExpired);

    // check operator
    distributor.authorize_claim_and_stake(&ctx.accounts.operator)?;

    let escrow = &ctx.accounts.escrow;
    require!(escrow.is_max_lock, ErrorCode::EscrowIsNotMaxLock);

    let activation_handler = distributor.get_activation_handler()?;
    activation_handler.validate_claim()?;

    distributor.num_nodes_claimed = distributor
        .num_nodes_claimed
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticError)?;

    require!(
        distributor.num_nodes_claimed <= distributor.max_num_nodes,
        ErrorCode::MaxNodesExceeded
    );

    let claimant_account = &ctx.accounts.claimant;

    // Verify the merkle proof.
    let node = hashv(&[
        &claimant_account.key().to_bytes(),
        &amount_unlocked.to_le_bytes(),
        &amount_locked.to_le_bytes(),
    ]);

    let node = hashv(&[LEAF_PREFIX, &node.to_bytes()]);

    let verify_res = verify_partial_merkle(
        partial_merkle_tree.root,
        node.to_bytes(),
        partial_merkle_tree.depth,
        initial_index,
        proof,
        partial_merkle_tree.nodes.clone(),
    );
    require!(verify_res, ErrorCode::InvalidProof);
    let mut claim_status = ctx.accounts.claim_status.load_init()?;

    // Seed initial values
    claim_status.distributor = ctx.accounts.distributor.key();
    claim_status.claimant = claimant_account.key();
    claim_status.locked_amount = amount_locked;
    claim_status.unlocked_amount = amount_unlocked;
    claim_status.locked_amount_withdrawn = 0;
    claim_status.closable = distributor.closable;
    claim_status.admin = distributor.admin;

    claim_status.bonus_amount =
        distributor.get_bonus_for_a_claimaint(amount_unlocked, &activation_handler)?;

    let amount_with_bonus = claim_status.get_total_unlocked_amount()?;

    distributor.total_amount_claimed = distributor
        .total_amount_claimed
        .checked_add(amount_with_bonus)
        .ok_or(ErrorCode::ArithmeticError)?;

    distributor.accumulate_bonus(claim_status.bonus_amount)?;

    require!(
        distributor.total_amount_claimed <= distributor.max_total_claim,
        ErrorCode::ExceededMaxClaim
    );

    // Note: might get truncated, do not rely on
    msg!(
        "Created new claim with locked {}, unlocked {} and bonus {} with lockup start:{} end:{}",
        claim_status.locked_amount,
        claim_status.unlocked_amount,
        claim_status.bonus_amount,
        distributor.start_ts,
        distributor.end_ts,
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
    voter::cpi::increase_locked_amount(cpi_ctx, amount_with_bonus)?;

    emit!(NewClaimEvent {
        claimant: claimant_account.key(),
        timestamp: Clock::get()?.unix_timestamp
    });

    Ok(())
}
