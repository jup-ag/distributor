use anchor_lang::prelude::*;
use crate::error::ErrorCode::ArithmeticError;
use crate::math::SafeMath;

pub const CLAIM_STATUS_DISC: [u8; 8] = [22,
    183,
    249,
    157,
    247,
    95,
    150,
    96];

#[inline(always)]
fn cs_body_ro<'a>(data: &'a [u8]) -> Result<&'a [u8]> {
    if data.len() < 8 { return Err(ProgramError::InvalidAccountData.into()); }
    if &data[..8] != &CLAIM_STATUS_DISC { return Err(ProgramError::InvalidAccountData.into()); }
    Ok(&data[8..])
}
#[inline(always)]
fn cs_body_rw<'a>(data: &'a mut [u8]) -> Result<&'a mut [u8]> {
    if data.len() < 8 { return Err(ProgramError::InvalidAccountData.into()); }
    if &data[..8] != &CLAIM_STATUS_DISC { return Err(ProgramError::InvalidAccountData.into()); }
    Ok(&mut data[8..])
}

// ------------------------------------------------------------------
// ZERO-COPY PinoClaimStatus
// - POD layout (bool -> u8 + padding)
// - Can be read/written in place via cs_load/cs_load_mut
// ------------------------------------------------------------------
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PinoClaimStatus {
    /// Authority that claimed the tokens.
    pub claimant: Pubkey,
    /// Locked amount
    pub locked_amount: u64,
    /// Locked amount withdrawn
    pub locked_amount_withdrawn: u64,
    /// Unlocked amount (total entitlement, not yet net of withdrawals)
    pub unlocked_amount: u64,
    /// indicate that whether admin can close this account (POD flag)
    pub closable_u8: u8,
    pub _pad0: [u8; 7], // align next Pubkey
    /// admin of merkle tree, store for testing purpose
    pub admin: Pubkey,
}

impl PinoClaimStatus {
    #[inline] pub fn closable(&self) -> bool { self.closable_u8 != 0 }
    #[inline] pub fn set_closable(&mut self, v: bool) { self.closable_u8 = v as u8; }

    /// For allocations: body (without discriminator) and total (with 8-byte discriminator).
    pub const BODY_LEN: usize = core::mem::size_of::<PinoClaimStatus>();
    pub const TOTAL_LEN: usize = 8 + core::mem::size_of::<PinoClaimStatus>();
    /// Backward-compatible alias if other code expects LEN
    pub const LEN: usize = Self::TOTAL_LEN;

    /// Returns amount withdrawable at `curr_ts`, factoring in prior withdrawals.
    /// payout = unlocked_amount(curr_ts) - locked_amount_withdrawn
    #[allow(clippy::result_large_err)]
    pub fn amount_withdrawable(&self, curr_ts: i64, start_ts: i64, end_ts: i64) -> Result<u64> {
        let amount = self
            .unlocked_amount(curr_ts, start_ts, end_ts)?
            .checked_sub(self.locked_amount_withdrawn)
            .ok_or(ArithmeticError)?;
        Ok(amount)
    }

    /// Linearly vested unlocked portion of `locked_amount` as of `curr_ts`.
    #[allow(clippy::result_large_err)]
    pub fn unlocked_amount(&self, curr_ts: i64, start_ts: i64, end_ts: i64) -> Result<u64> {
        if curr_ts >= start_ts {
            if curr_ts >= end_ts {
                Ok(self.locked_amount)
            } else {
                let time_into_unlock = curr_ts.checked_sub(start_ts).ok_or(ArithmeticError)?;
                let total_unlock_time = end_ts.checked_sub(start_ts).ok_or(ArithmeticError)?;

                let amount = ((time_into_unlock as u128)
                    .checked_mul(self.locked_amount as u128)
                    .ok_or(ArithmeticError)?)
                    .checked_div(total_unlock_time as u128)
                    .ok_or(ArithmeticError)? as u64;

                Ok(amount)
            }
        } else {
            Ok(0)
        }
    }
}

// ------------------------------------------------------------------
// Zero-copy accessors (read-only / mutable) and initializer
// ------------------------------------------------------------------

#[inline(always)]
pub fn cs_load<'a>(ai: &'a AccountInfo) -> Result<&'a PinoClaimStatus> {
    let data = ai.try_borrow_data()?;
    let body = cs_body_ro(&data)?;
    if body.len() < PinoClaimStatus::BODY_LEN {
        return Err(ProgramError::InvalidAccountData.into());
    }
    // SAFETY: repr(C) + POD; bounds checked above
    Ok(unsafe { &*(body.as_ptr() as *const PinoClaimStatus) })
}

#[inline(always)]
pub fn cs_load_mut<'a>(ai: &'a AccountInfo) -> Result<&'a mut PinoClaimStatus> {
    let mut data = ai.try_borrow_mut_data()?;
    let body = cs_body_rw(&mut data)?;
    if body.len() < PinoClaimStatus::BODY_LEN {
        return Err(ProgramError::InvalidAccountData.into());
    }
    Ok(unsafe { &mut *(body.as_mut_ptr() as *mut PinoClaimStatus) })
}

/// Initialize a freshly created account's data section:
/// writes the discriminator and zeroes the body.
#[inline(always)]
pub fn cs_init_zero(ai: &AccountInfo) -> Result<()> {
    let mut data = ai.try_borrow_mut_data()?;
    if data.len() < 8 { return Err(ProgramError::InvalidAccountData.into()); }
    data[..8].copy_from_slice(&CLAIM_STATUS_DISC);

    let body_len = PinoClaimStatus::BODY_LEN;
    if data.len() < 8 + body_len { return Err(ProgramError::InvalidAccountData.into()); }
    let body = &mut data[8..8 + body_len];
    for b in body.iter_mut() { *b = 0; }
    Ok(())
}
