use anchor_lang::{
    context::Context, prelude::*, solana_program::hash::hashv, system_program::System, Accounts,
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
        claim_status::ClaimStatus, claimed_event::NewClaimEvent,
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

    /// Claim status PDA
    #[account(
        init,
        seeds = [
            b"ClaimStatus".as_ref(),
            claimant.key().to_bytes().as_ref(),
            distributor.key().to_bytes().as_ref()
        ],
        bump,
        space = 8 + ClaimStatus::LEN,
        payer = claimant,
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

    /// The [System] program.
    pub system_program: Program<'info, System>,
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
pub fn handle_new_claim(
    ctx: Context<NewClaim>,
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

    let claimant_account = &ctx.accounts.claimant;

    // Verify the merkle proof.
    let node = hashv(&[
        &claimant_account.key().to_bytes(),
        &amount_unlocked.to_le_bytes(),
        &amount_locked.to_le_bytes(),
    ]);

    let node = hashv(&[LEAF_PREFIX, &node.to_bytes()]);

    require!(
        verify(proof, distributor.root, node.to_bytes()),
        ErrorCode::InvalidProof
    );

    let mut claim_status = ctx.accounts.claim_status.load_init()?;

    // Seed initial values
    claim_status.distributor = ctx.accounts.distributor.key();
    claim_status.claimant = claimant_account.key();
    claim_status.locked_amount = amount_locked;
    claim_status.locked_amount_withdrawn = 0;
    claim_status.closable = distributor.closable;
    claim_status.admin = distributor.admin;

    claim_status.unlocked_amount = amount_unlocked;
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
        "Created new claim with locked {}, unlocked {} and bonus {} with lockup start:{} end:{}, activation_point {} current_point {}",
        claim_status.locked_amount,
        claim_status.unlocked_amount,
        claim_status.bonus_amount,
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
