use super::p_utils::{read_token_amount, read_token_mint_owner};

use anchor_lang::{prelude::*, solana_program::system_program};
use anchor_spl::token;

use crate::{
    error::ErrorCode,
    state::pino_distributor::{md_load, md_load_mut},
};

/// Hot path clawback implementation.
pub fn p_handle_clawback<'info>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'info>],
    ix_data: &[u8],
) -> Result<()> {
    if ix_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    let [
        distributor_ai,
        from_ai,
        to_ai,
        claimant_ai,
        system_program_ai,
        token_program_ai,
        ..
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    if token_program_ai.key() != token::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if system_program_ai.key() != system_program::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if !claimant_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }
    if distributor_ai.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if from_ai.owner != token_program_ai.key || to_ai.owner != token_program_ai.key {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let curr_ts = Clock::get()?.unix_timestamp;
    let (
        base,
        mint,
        version_le,
        bump_arr,
        token_vault,
        clawback_receiver,
    ) = {
        let distributor = md_load(distributor_ai)?;
        require!(
            !distributor.clawed_back(),
            ErrorCode::ClawbackAlreadyClaimed
        );
        require!(
            curr_ts >= distributor.clawback_start_ts,
            ErrorCode::ClawbackBeforeStart
        );
        (
            distributor.base,
            distributor.mint,
            distributor.version.to_le_bytes(),
            [distributor.bump],
            distributor.token_vault,
            distributor.clawback_receiver,
        )
    };

    if *from_ai.key != token_vault {
        return Err(ProgramError::InvalidAccountData.into());
    }
    if *to_ai.key != clawback_receiver {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let (from_mint, from_owner) = read_token_mint_owner(from_ai)?;
    if from_mint != mint || from_owner != *distributor_ai.key {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let (to_mint, _) = read_token_mint_owner(to_ai)?;
    if to_mint != mint {
        return Err(ProgramError::InvalidAccountData.into());
    }

    let amount = read_token_amount(from_ai)?;

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

    let distributor = md_load_mut(distributor_ai)?;
    distributor.set_clawed_back(true);

    Ok(())
}
