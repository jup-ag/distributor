#![allow(clippy::too_many_arguments)]

use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
    AccountDeserialize, AccountSerialize,
};
use anchor_spl::token;
use jito_merkle_verify::verify as merkle_verify;

use crate::{
    error::ErrorCode,
    math::SafeMath,
    state::{
        claim_status::ClaimStatus,
        claimed_event::NewClaimEvent,
        merkle_distributor::MerkleDistributor,
    },
};

const LEAF_PREFIX: &[u8] = &[0];

/// Parse args for `new_claim` after the 8-byte discriminator:
/// u64 amount_unlocked | u64 amount_locked | u32 proof_len | proof[[u8;32]; N]
fn parse_new_claim_args(ix_data: &[u8]) -> Result<(u64, u64, Vec<[u8; 32]>)> {
    // Use plain checks for ProgramError to avoid require!-with-ProgramError hassles
    if ix_data.len() < 8 + 8 + 8 + 4 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    let (_, rest) = ix_data.split_at(8);
    let (un_bytes, rest) = rest.split_at(8);
    let (lk_bytes, rest) = rest.split_at(8);
    let (plen_bytes, rest) = rest.split_at(4);

    let amount_unlocked = u64::from_le_bytes(un_bytes.try_into().unwrap());
    let amount_locked   = u64::from_le_bytes(lk_bytes.try_into().unwrap());
    let proof_len       = u32::from_le_bytes(plen_bytes.try_into().unwrap()) as usize;

    if rest.len() != proof_len * 32 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    let mut proof = Vec::with_capacity(proof_len);
    for i in 0..proof_len {
        let start = i * 32;
        proof.push(rest[start..start + 32].try_into().unwrap());
    }
    Ok((amount_unlocked, amount_locked, proof))
}

pub fn p_handle_new_claim<'info>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'info>],
    ix_data: &[u8],
) -> Result<()> {
    // Accounts (same order as your Anchor NewClaim):
    // 0 distributor (w)
    // 1 claim_status (w)
    // 2 from (vault ATA, w)
    // 3 to   (claimant ATA, w)
    // 4 claimant (signer)
    // 5 token_program
    // 6 system_program
    let [distributor_ai, claim_status_ai, from_ai, to_ai, claimant_ai, token_program_ai, system_program_ai, ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    if !claimant_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }
    if token_program_ai.key != &anchor_spl::token::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if system_program_ai.key != &System::id() {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let (amount_unlocked, amount_locked, proof) = parse_new_claim_args(ix_data)?;

    let dist_data_ref = distributor_ai.try_borrow_data()?;
    let mut bytes: &[u8] = &dist_data_ref;
    let mut distributor: MerkleDistributor = AccountDeserialize::try_deserialize(&mut bytes)?;

    require!(!distributor.clawed_back, ErrorCode::ClaimExpired);

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

    let inner = anchor_lang::solana_program::hash::hashv(&[
        &claimant_ai.key.to_bytes(),
        &amount_unlocked.to_le_bytes(),
        &amount_locked.to_le_bytes(),
    ]);
    let leaf = anchor_lang::solana_program::hash::hashv(&[LEAF_PREFIX, &inner.to_bytes()]);
    require!(merkle_verify(proof, distributor.root, leaf.to_bytes()), ErrorCode::InvalidProof);

    let (expected_claim_status, _bump) = Pubkey::find_program_address(
        &[b"ClaimStatus", claimant_ai.key.as_ref(), distributor_ai.key.as_ref()],
        program_id,
    );
    if claim_status_ai.key != &expected_claim_status {
        return Err(ProgramError::InvalidSeeds.into());
    }

    // Create ClaimStatus if needed (Anchor init)
    if claim_status_ai.lamports() == 0 || claim_status_ai.owner != program_id {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(ClaimStatus::LEN);
        let create_ix = system_instruction::create_account(
            claimant_ai.key,             // payer (signer)
            &expected_claim_status,      // new account
            lamports,
            ClaimStatus::LEN as u64,
            program_id,                  // owner = this program
        );
        invoke_signed(
            &create_ix,
            &[
                claimant_ai.clone(),
                claim_status_ai.clone(),
                system_program_ai.clone(),
            ],
            &[], // no PDA signer; payer is a signer
        )?;
    }

    // Compute bonus & unlocked_with_bonus
    let bonus = distributor.get_bonus_for_a_claimaint(amount_unlocked, &activation_handler)?;
    let unlocked_with_bonus = amount_unlocked
        .safe_add(bonus)
        .map_err(|_| ErrorCode::ArithmeticError)?;

    // Serialize ClaimStatus into the account (writes discriminator first)
    {
        let mut cs_data = claim_status_ai.try_borrow_mut_data()?;
        let claim_status = ClaimStatus {
            claimant: *claimant_ai.key,
            locked_amount: amount_locked,
            locked_amount_withdrawn: 0,
            closable: distributor.closable,
            admin: distributor.admin,
            unlocked_amount: unlocked_with_bonus,
        };
        let mut write_cursor: &mut [u8] = &mut cs_data;
        AccountSerialize::try_serialize(&claim_status, &mut write_cursor)?;
    }

    // SPL Token transfer via anchor_spl (authority = distributor PDA)
    // Seeds: ["MerkleDistributor", base, mint, version_le, bump]
    let seeds = [
        b"MerkleDistributor".as_ref(),
        distributor.base.as_ref(),
        distributor.mint.as_ref(),
        &distributor.version.to_le_bytes(),
        &[distributor.bump],
    ];
    token::transfer(
        CpiContext::new(
            token_program_ai.clone(), // Program<'_, Token>::to_account_info()
            token::Transfer {
                from: from_ai.clone(),
                to: to_ai.clone(),
                authority: distributor_ai.clone(),
            },
        )
            .with_signer(&[&seeds]),
        unlocked_with_bonus,
    )?;

    // Update distributor state and persist
    distributor.total_amount_claimed = distributor
        .total_amount_claimed
        .checked_add(unlocked_with_bonus)
        .ok_or(ErrorCode::ArithmeticError)?;
    distributor.accumulate_bonus(bonus)?;
    require!(
        distributor.total_amount_claimed <= distributor.max_total_claim,
        ErrorCode::ExceededMaxClaim
    );

    drop(dist_data_ref); // release the read borrow
    let mut d_mut = distributor_ai.try_borrow_mut_data()?;
    let mut write_cursor: &mut [u8] = &mut d_mut;
    AccountSerialize::try_serialize(&distributor, &mut write_cursor)?;

    // Event + log
    let now = Clock::get()?.unix_timestamp;
    msg!(
        "Created new claim (fast) locked {} unlocked {} bonus {} start {} end {} activation_point {} current_point {}",
        amount_locked,
        amount_unlocked,
        bonus,
        distributor.start_ts,
        distributor.end_ts,
        activation_handler.activation_point,
        activation_handler.curr_point,
    );
    emit!(NewClaimEvent {
        claimant: *claimant_ai.key,
        timestamp: now
    });

    Ok(())
}
