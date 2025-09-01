use anchor_lang::{
    context::Context, prelude::*, solana_program::hash::hashv, Accounts,
    Key, Result,
};
use anchor_spl::{
    token,
    token::{Token, TokenAccount},
};
use jito_merkle_verify::verify;

use crate::{
    error::ErrorCode,
    state::{
        claimed_event::NewClaimEvent,
        merkle_distributor::MerkleDistributor,
    },
};

// We need to discern between leaf and intermediate nodes to prevent trivial second
// pre-image attacks.
// https://flawed.net.nz/2018/02/21/attacking-merkle-trees-with-a-second-preimage-attack
pub const LEAF_PREFIX: &[u8] = &[0];

/// [merkle_distributor::new_claim] accounts.
#[derive(Accounts)]
pub struct NewClaim<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: AccountLoader<'info, MerkleDistributor>,

    // REMOVED: Claim status PDA (no per-claimer account)
    // #[account(…)]
    // pub claim_status: AccountLoader<'info, ClaimStatus>,

    /// Distributor ATA containing the tokens to distribute.
    #[account(
        mut,
        associated_token::mint = distributor.load()?.mint,
        associated_token::authority = distributor.key(),
        address = distributor.load()?.token_vault
    )]
    pub from: Account<'info, TokenAccount>,

    /// Account to send the claimed tokens to.
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    /// Who is claiming the tokens.
    #[account(mut)]
    pub claimant: Signer<'info>,

    /// operator
    pub operator: Option<Signer<'info>>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,

}

/// Initializes a new claim from the [MerkleDistributor].
/// 1. Increments num_nodes_claimed by 1
/// 2. Verifies proof (leaf commits to claimant, index, amounts)
/// 3. Marks index as claimed in distributor.claimed_bitmap///
/// 4. Transfers claim_status.unlocked_amount to the claimant
/// 5. Increments total_amount_claimed by claim_status.unlocked_amount
/// CHECK:
///     1. The claim window has not expired and the distributor has not been clawed back
///     2. The claimant is the owner of the to account
///     3. Num nodes claimed is less than max_num_nodes
///     4. The merkle proof is valid
#[allow(clippy::result_large_err)]
pub fn handle_new_claim(
    ctx: Context<NewClaim>,
    index: u32,                    // NEW: leaf index for bitmap
    amount_unlocked: u64,
    amount_locked: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let mut distributor = ctx.accounts.distributor.load_mut()?;

    require!(!distributor.clawed_back(), ErrorCode::ClaimExpired);

    // check operator
    distributor.authorize_claim(&ctx.accounts.operator)?;

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

    // Index / bitmap checks
    require!(
        (index as u64) < distributor.max_num_nodes,
        ErrorCode::IndexOutOfRange
    );
    require!(
        !distributor.claimed_bitmap.is_set(index),
        ErrorCode::AlreadyClaimed
    );

    let claimant_account = &ctx.accounts.claimant;

    // Construct leaf: include index to bind the unique slot in the bitmap.
    // Format: hash( LEAF_PREFIX || claimant || index_be || amount_unlocked_be || amount_locked_be )
    let node_inner = hashv(&[
        &claimant_account.key().to_bytes(),
        &index.to_be_bytes(),
        &amount_unlocked.to_be_bytes(),
        &amount_locked.to_be_bytes(),
    ]);
    let node = hashv(&[LEAF_PREFIX, &node_inner.to_bytes()]);

    // Verify the merkle proof.
    require!(
        verify(proof, distributor.root, node.to_bytes()),
        ErrorCode::InvalidProof
    );

    // Compute bonus and payout
    let bonus_amount =
        distributor.get_bonus_for_a_claimaint(amount_unlocked, &activation_handler)?;
    let amount_with_bonus = amount_unlocked
        .checked_add(bonus_amount)
        .ok_or(ErrorCode::ArithmeticError)?;

    // State updates BEFORE transfer (prevent races)
    distributor.claimed_bitmap.set(index);
    distributor.num_nodes_claimed = distributor
        .num_nodes_claimed
        .checked_add(1)
        .ok_or(ErrorCode::ArithmeticError)?;

    require!(
        distributor.num_nodes_claimed <= distributor.max_num_nodes,
        ErrorCode::MaxNodesExceeded
    );

    distributor.total_amount_claimed = distributor
        .total_amount_claimed
        .checked_add(amount_with_bonus)
        .ok_or(ErrorCode::ArithmeticError)?;

    distributor.accumulate_bonus(bonus_amount)?;
    require!(
        distributor.total_amount_claimed <= distributor.max_total_claim,
        ErrorCode::ExceededMaxClaim
    );

    // Note: might get truncated, do not rely on
    msg!(
        "New claim idx:{} locked {} unlocked {} bonus {} lockup start:{} end:{}, activation_point {} current_point {}",
        index,
        amount_locked,
        amount_unlocked,
        bonus_amount,
        distributor.start_ts,
        distributor.end_ts,
        activation_handler.activation_point,
        activation_handler.curr_point,
    );

    let signer = distributor.signer();
    drop(distributor);
    let seeds = signer.seeds();

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
        amount_with_bonus,
    )?;

    emit!(NewClaimEvent {
        claimant: claimant_account.key(),
        timestamp: Clock::get()?.unix_timestamp
    });

    Ok(())
}
