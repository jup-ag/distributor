#![allow(clippy::too_many_arguments)]

use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};
use anchor_spl::token;
use jito_merkle_verify::verify as merkle_verify;

use crate::{
    error::ErrorCode,
    math::SafeMath,
    state::{
        claimed_event::NewClaimEvent,
        pino_claim_status::{cs_init_zero, cs_load_mut, PinoClaimStatus},
        pino_distributor::{md_load, md_load_mut, PinoMerkleDistributor},
    },
};

const LEAF_PREFIX: &[u8] = &[0];

/// Parse args for `new_claim` after the 8-byte discriminator:
/// u64 amount_unlocked | u64 amount_locked | u32 proof_len | proof[[u8;32]; N]
fn parse_new_claim_args(ix_data: &[u8]) -> Result<(u64, u64, Vec<[u8; 32]>)> {
    if ix_data.len() < 8 + 8 + 8 + 4 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    let (_, rest) = ix_data.split_at(8);
    let (un_bytes, rest) = rest.split_at(8);
    let (lk_bytes, rest) = rest.split_at(8);
    let (plen_bytes, rest) = rest.split_at(4);

    let amount_unlocked = u64::from_le_bytes(un_bytes.try_into().unwrap());
    let amount_locked = u64::from_le_bytes(lk_bytes.try_into().unwrap());
    let proof_len = u32::from_le_bytes(plen_bytes.try_into().unwrap()) as usize;

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
    // Accounts:
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
    if token_program_ai.key() != anchor_spl::token::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if system_program_ai.key() != System::id() {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let (amount_unlocked, amount_locked, proof) = parse_new_claim_args(ix_data)?;

    // Distributor: read-only checks (short borrow)
    {
        let d = md_load(distributor_ai)?;
        require!(!d.clawed_back(), ErrorCode::ClaimExpired);

        let activation_handler = d.get_activation_handler()?;
        activation_handler.validate_claim()?;
    }

    // Bump num_nodes_claimed (short mutable borrow)
    {
        let d = md_load_mut(distributor_ai)?;
        d.num_nodes_claimed = d
            .num_nodes_claimed
            .checked_add(1)
            .ok_or(ErrorCode::ArithmeticError)?;
        require!(
            d.num_nodes_claimed <= d.max_num_nodes,
            ErrorCode::MaxNodesExceeded
        );
    }

    // Merkle proof
    let leaf_inner = anchor_lang::solana_program::hash::hashv(&[
        claimant_ai.key.as_ref(),
        &amount_unlocked.to_le_bytes(),
        &amount_locked.to_le_bytes(),
    ]);
    let leaf = anchor_lang::solana_program::hash::hashv(&[LEAF_PREFIX, &leaf_inner.to_bytes()]);
    {
        let d = md_load(distributor_ai)?;
        require!(
            merkle_verify(proof, d.root, leaf.to_bytes()),
            ErrorCode::InvalidProof
        );
    }

    // ClaimStatus PDA & (lazy) init
    let (expected_claim_status, _bump) = Pubkey::find_program_address(
        &[
            b"ClaimStatus",
            claimant_ai.key.as_ref(),
            distributor_ai.key.as_ref(),
        ],
        program_id,
    );
    if claim_status_ai.key() != expected_claim_status {
        return Err(ProgramError::InvalidSeeds.into());
    }

    if claim_status_ai.lamports() == 0 || claim_status_ai.owner != program_id {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(PinoClaimStatus::TOTAL_LEN);
        let create_ix = system_instruction::create_account(
            claimant_ai.key,        // payer
            &expected_claim_status, // new account
            lamports,
            PinoClaimStatus::TOTAL_LEN as u64,
            program_id,
        );
        invoke_signed(
            &create_ix,
            &[
                claimant_ai.clone(),
                claim_status_ai.clone(),
                system_program_ai.clone(),
            ],
            &[],
        )?;
        cs_init_zero(claim_status_ai)?;
    }

    // Compute bonus, write ClaimStatus in place
    let (
        bonus,
        unlocked_with_bonus,
        closable,
        admin,
        base,
        mint,
        version_le,
        bump_arr,
        activation_point,
        current_point,
    ) = {
        let d = md_load(distributor_ai)?;
        let activation_handler = d.get_activation_handler()?;
        let bonus = d.get_bonus_for_a_claimaint(amount_unlocked, &activation_handler)?;
        let unlocked_with_bonus = amount_unlocked
            .safe_add(bonus)
            .map_err(|_| ErrorCode::ArithmeticError)?;
        // COPY out everything needed for seeds so we don't borrow `d`
        let base: Pubkey = d.base;
        let mint: Pubkey = d.mint;
        let version_le: [u8; 8] = d.version.to_le_bytes();
        let bump_arr: [u8; 1] = [d.bump];
        let activation_point = activation_handler.activation_point;
        let current_point = activation_handler.curr_point;
        (
            bonus,
            unlocked_with_bonus,
            d.closable(),
            d.admin,
            base,
            mint,
            version_le,
            bump_arr,
            activation_point,
            current_point,
        )
    };

    {
        let cs = cs_load_mut(claim_status_ai)?;
        // initialize/update fields (in place)
        cs.claimant = *claimant_ai.key;
        cs.locked_amount = amount_locked;
        cs.locked_amount_withdrawn = 0;
        cs.unlocked_amount = unlocked_with_bonus;
        cs.set_closable(closable);
        cs.admin = admin;
    }

    // later, right before CPI:
    let seeds: [&[u8]; 5] = [
        b"MerkleDistributor".as_ref(),
        base.as_ref(),
        mint.as_ref(),
        &version_le,
        &bump_arr,
    ];

    token::transfer(
        CpiContext::new(
            token_program_ai.clone(),
            token::Transfer {
                from: from_ai.clone(),
                to: to_ai.clone(),
                authority: distributor_ai.clone(),
            },
        )
        .with_signer(&[&seeds]),
        unlocked_with_bonus,
    )?;

    // Update distributor totals (short mutable borrow)
    let d = md_load_mut(distributor_ai)?;
    d.total_amount_claimed = d
        .total_amount_claimed
        .checked_add(unlocked_with_bonus)
        .ok_or(ErrorCode::ArithmeticError)?;
    d.accumulate_bonus(bonus)?;
    require!(
        d.total_amount_claimed <= d.max_total_claim,
        ErrorCode::ExceededMaxClaim
    );

    msg!(
        "Created new claim (fast) locked {} unlocked {} bonus {} start {} end {} activation_point {} current_point {}",
        amount_locked,
        amount_unlocked,
        bonus,
        d.start_ts,
        d.end_ts,
        activation_point,
        current_point,
    );

    emit!(NewClaimEvent {
        claimant: *claimant_ai.key,
        timestamp: Clock::get()?.unix_timestamp
    });

    Ok(())
}
