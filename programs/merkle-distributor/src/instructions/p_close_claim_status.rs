use super::p_utils::close_account_to_destination;

use anchor_lang::prelude::*;

use crate::{
    error::ErrorCode,
    state::{
        claim_status::ClaimStatus,
        pino_claim_status::{cs_load, PinoClaimStatus},
    },
};

fn load_meta(ai: &AccountInfo) -> Result<(Pubkey, Pubkey, bool)> {
    let data = ai.try_borrow_data()?;
    let len = data.len();
    if len == PinoClaimStatus::TOTAL_LEN {
        drop(data);
        let cs = cs_load(ai)?;
        Ok((cs.claimant, cs.admin, cs.closable()))
    } else {
        let mut slice: &[u8] = &data;
        let cs = ClaimStatus::try_deserialize(&mut slice)?;
        Ok((cs.claimant, cs.admin, cs.closable))
    }
}

pub fn p_handle_close_claim_status<'info>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'info>],
    ix_data: &[u8],
) -> Result<()> {
    if ix_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData.into());
    }

    let [claim_status_ai, claimant_ai, admin_ai, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    if claim_status_ai.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }
    if !admin_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }

    let (claimant_pk, admin_pk, closable) = load_meta(claim_status_ai)?;
    require!(closable, ErrorCode::CannotCloseClaimStatus);
    require_keys_eq!(claimant_pk, claimant_ai.key(), ErrorCode::OwnerMismatch);
    require_keys_eq!(admin_pk, admin_ai.key(), ErrorCode::OwnerMismatch);

    msg!("Closing claim status via fast path");
    close_account_to_destination(claim_status_ai, claimant_ai)?;

    Ok(())
}
