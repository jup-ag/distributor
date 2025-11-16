use anchor_lang::prelude::*;
use anchor_spl::token;

use crate::{
    error::ErrorCode,
    state::{
        claimed_event::ClaimedEvent,
        pino_claim_status::{cs_load, cs_load_mut},
        pino_distributor::{md_load, md_load_mut},
    },
};

#[inline(always)]
fn read_token_account_mint_owner(ai: &AccountInfo) -> Result<(Pubkey, Pubkey)> {
    let data = ai.try_borrow_data()?;
    if data.len() < 64 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let mut mint_bytes = [0u8; 32];
    mint_bytes.copy_from_slice(&data[0..32]);
    let mut owner_bytes = [0u8; 32];
    owner_bytes.copy_from_slice(&data[32..64]);
    Ok((
        Pubkey::new_from_array(mint_bytes),
        Pubkey::new_from_array(owner_bytes),
    ))
}

/// Hot-path implementation of [ClaimLocked].
pub fn p_handle_claim_locked<'info>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'info>],
    ix_data: &[u8],
) -> Result<()> {
    if ix_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    let [
        distributor_ai,
        claim_status_ai,
        from_ai,
        to_ai,
        claimant_ai,
        token_program_ai,
        ..
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    if token_program_ai.key() != token::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if !claimant_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }
    if distributor_ai.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if claim_status_ai.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if from_ai.owner != token_program_ai.key {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if to_ai.owner != token_program_ai.key {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let (expected_claim_status, _) = Pubkey::find_program_address(
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

    let curr_ts = Clock::get()?.unix_timestamp;

    let (
        start_ts,
        end_ts,
        base,
        mint,
        version_le,
        bump_arr,
        token_vault,
        total_claimed,
    ) = {
        let distributor = md_load(distributor_ai)?;
        require!(!distributor.clawed_back(), ErrorCode::ClaimExpired);

        let activation_handler = distributor.get_activation_handler()?;
        activation_handler.validate_claim()?;

        (
            distributor.start_ts,
            distributor.end_ts,
            distributor.base,
            distributor.mint,
            distributor.version.to_le_bytes(),
            [distributor.bump],
            distributor.token_vault,
            distributor.total_amount_claimed,
        )
    };

    if *from_ai.key != token_vault {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let (from_mint, from_owner) = read_token_account_mint_owner(from_ai)?;
    if from_mint != mint || from_owner != *distributor_ai.key {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let (to_mint, to_owner) = read_token_account_mint_owner(to_ai)?;
    if to_owner != *claimant_ai.key {
        return Err(ErrorCode::OwnerMismatch.into());
    }
    if to_mint != mint {
        return Err(ProgramError::InvalidAccountData.into());
    }

    let amount = {
        let cs = cs_load(claim_status_ai)?;
        require_keys_eq!(cs.claimant, claimant_ai.key(), ErrorCode::OwnerMismatch);
        cs.amount_withdrawable(curr_ts, start_ts, end_ts)?
    };
    require!(amount > 0, ErrorCode::InsufficientUnlockedTokens);

    // Prepare seeds before mutable borrows.
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
        amount,
    )?;

    {
        let cs = cs_load_mut(claim_status_ai)?;
        cs.locked_amount_withdrawn = cs
            .locked_amount_withdrawn
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticError)?;
        require!(
            cs.locked_amount_withdrawn <= cs.locked_amount,
            ErrorCode::ExceededMaxClaim
        );
    }

    {
        let distributor = md_load_mut(distributor_ai)?;
        distributor.total_amount_claimed = total_claimed
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticError)?;
        require!(
            distributor.total_amount_claimed <= distributor.max_total_claim,
            ErrorCode::ExceededMaxClaim
        );
    }

    let remaining_seconds = if curr_ts < end_ts {
        end_ts - curr_ts
    } else {
        0
    };
    let days = remaining_seconds / (24 * 60 * 60);
    let seconds_after_days = remaining_seconds % (24 * 60 * 60);

    msg!(
        "Withdrew amount {} with {} days and {} seconds left in lockup",
        amount,
        days,
        seconds_after_days,
    );
    emit!(ClaimedEvent {
        claimant: *claimant_ai.key,
        amount,
    });

    Ok(())
}
