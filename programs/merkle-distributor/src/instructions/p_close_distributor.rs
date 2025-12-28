use super::p_utils::{close_account_to_destination, read_token_amount, read_token_mint_owner};

use anchor_lang::prelude::*;
use anchor_spl::token;

use crate::{
    error::ErrorCode,
    state::pino_distributor::md_load,
};

pub fn p_handle_close_distributor<'info>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'info>],
    ix_data: &[u8],
) -> Result<()> {
    if ix_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let [
        distributor_ai,
        token_vault_ai,
        admin_ai,
        destination_ai,
        token_program_ai,
        ..
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    if distributor_ai.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if token_program_ai.key() != token::ID {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if token_vault_ai.owner != token_program_ai.key
        || destination_ai.owner != token_program_ai.key
    {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if !admin_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }

    let (
        base,
        mint,
        version_le,
        bump_arr,
        admin_pubkey,
        token_vault_key,
        closable,
    ) = {
        let distributor = md_load(distributor_ai)?;
        (
            distributor.base,
            distributor.mint,
            distributor.version.to_le_bytes(),
            [distributor.bump],
            distributor.admin,
            distributor.token_vault,
            distributor.closable(),
        )
    };

    require!(closable, ErrorCode::CannotCloseDistributor);
    require_keys_eq!(admin_pubkey, admin_ai.key(), ErrorCode::OwnerMismatch);

    if *token_vault_ai.key != token_vault_key {
        return Err(ProgramError::InvalidAccountData.into());
    }

    let (vault_mint, vault_owner) = read_token_mint_owner(token_vault_ai)?;
    if vault_mint != mint || vault_owner != *distributor_ai.key {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let (dest_mint, _) = read_token_mint_owner(destination_ai)?;
    if dest_mint != mint {
        return Err(ProgramError::InvalidAccountData.into());
    }

    let amount = read_token_amount(token_vault_ai)?;

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
                from: token_vault_ai.clone(),
                to: destination_ai.clone(),
                authority: distributor_ai.clone(),
            },
        )
        .with_signer(&[&seeds]),
        amount,
    )?;

    close_account_to_destination(distributor_ai, admin_ai)?;

    Ok(())
}
