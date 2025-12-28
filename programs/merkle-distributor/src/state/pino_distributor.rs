use crate::error::ErrorCode;
use crate::math::safe_math::SafeMath;
use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const MERKLE_DISTRIBUTOR_DISC: [u8; 8] = [77, 119, 139, 70, 84, 247, 12, 26];

/// ---------------------------------------------------------------------------
/// Helpers to check discriminator and slice body
/// ---------------------------------------------------------------------------
#[inline(always)]
fn body_ro<'a>(data: &'a [u8], disc: &[u8; 8]) -> Result<&'a [u8]> {
    if data.len() < 8 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    if &data[..8] != disc {
        return Err(ProgramError::InvalidAccountData.into());
    }
    Ok(&data[8..])
}

#[inline(always)]
fn body_rw<'a>(data: &'a mut [u8], disc: &[u8; 8]) -> Result<&'a mut [u8]> {
    if data.len() < 8 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    if &data[..8] != disc {
        return Err(ProgramError::InvalidAccountData.into());
    }
    Ok(&mut data[8..])
}

/// ---------------------------------------------------------------------------
/// Activation type
/// ---------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the activation
pub enum ActivationType {
    Slot,
    Timestamp,
}

/// ---------------------------------------------------------------------------
/// Zero-copy AirdropBonus (POD)
/// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PinoAirdropBonus {
    /// total bonus
    pub total_bonus: u64,
    /// vesting duration
    pub vesting_duration: u64,
    /// total claimed bonus
    pub total_claimed_bonus: u64,
}

/// ---------------------------------------------------------------------------
/// Zero-copy MerkleDistributor (POD)
/// NOTE: This is a C layout for in-place reads/writes. It is NOT Borsh.
///       Use md_load/md_load_mut below to access it safely.
/// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Debug, Default)]
pub struct PinoMerkleDistributor {
    /// Bump seed.
    pub bump: u8,
    pub _pad_bump: [u8; 7], // align next u64

    /// Version of the airdrop
    pub version: u64,

    /// The 256-bit merkle root.
    pub root: [u8; 32],

    /// [Mint] of the token to be distributed.
    pub mint: Pubkey,
    /// base key of distributor.
    pub base: Pubkey,
    /// Token Address of the vault
    pub token_vault: Pubkey,

    /// Maximum number of tokens that can ever be claimed from this [PinoMerkleDistributor].
    pub max_total_claim: u64,
    /// Maximum number of nodes in [PinoMerkleDistributor].
    pub max_num_nodes: u64,
    /// Total amount of tokens that have been claimed.
    pub total_amount_claimed: u64,
    /// Number of nodes that have been claimed.
    pub num_nodes_claimed: u64,

    /// Lockup time start (Unix Timestamp)
    pub start_ts: i64,
    /// Lockup time end (Unix Timestamp)
    pub end_ts: i64,
    /// Clawback start (Unix Timestamp)
    pub clawback_start_ts: i64,

    /// Clawback receiver
    pub clawback_receiver: Pubkey,
    /// Admin wallet
    pub admin: Pubkey,

    /// Whether or not the distributor has been clawed back (POD flag)
    pub clawed_back_u8: u8,
    pub _pad_clawed: [u8; 7], // align u64 below

    /// this merkle tree is activated from this slot or timestamp
    pub activation_point: u64,

    /// indicate that whether admin can close this pool, for testing purpose (POD flag)
    pub closable_u8: u8,
    pub _pad_closable: [u8; 7], // align AirdropBonus

    /// bonus multiplier / params
    pub airdrop_bonus: PinoAirdropBonus,

    /// activation type, 0 means slot, 1 means timestamp
    pub activation_type: u8,
    pub _pad_act_type: [u8; 7],

    /// Buffer 0
    pub buffer_0: [u8; 7],
    /// Buffer 1
    pub buffer_1: [u8; 32],
    /// Buffer 2
    pub buffer_2: [u8; 32],
}

/// --- convenience helpers for flags & sizes ---
impl PinoMerkleDistributor {
    #[inline] pub fn closable(&self) -> bool { self.closable_u8 != 0 }
    #[inline] pub fn set_closable(&mut self, v: bool) { self.closable_u8 = v as u8; }

    #[inline] pub fn clawed_back(&self) -> bool { self.clawed_back_u8 != 0 }
    #[inline] pub fn set_clawed_back(&mut self, v: bool) { self.clawed_back_u8 = v as u8; }

    pub const BODY_LEN: usize = core::mem::size_of::<PinoMerkleDistributor>();
    pub const TOTAL_LEN: usize = 8 + core::mem::size_of::<PinoMerkleDistributor>();
    /// Keep the old alias so other code compiles unchanged.
    pub const LEN: usize = Self::TOTAL_LEN;
}

/// ---------------------------------------------------------------------------
/// Activation logic (unchanged semantics, POD types)
/// ---------------------------------------------------------------------------
pub struct PinoActivationHandler {
    /// current slot or current timestamp
    pub curr_point: u64,
    /// activation slot or activation timestamp
    pub activation_point: u64,
    /// bonus multiplier
    pub airdrop_bonus: PinoAirdropBonus,
}

impl PinoActivationHandler {
    pub fn validate_claim(&self) -> Result<()> {
        require!(
            self.activation_point <= self.curr_point,
            ErrorCode::ClaimingIsNotStarted
        );
        Ok(())
    }

    pub fn get_bonus_for_a_claimaint(&self, max_bonus: u64) -> Result<u64> {
        let curr_point = self.curr_point;
        let start_point = self.activation_point;
        let end_point = self.airdrop_bonus.vesting_duration.safe_add(start_point)?;

        if curr_point >= start_point {
            if curr_point >= end_point {
                Ok(max_bonus)
            } else {
                let duration_into_unlock = curr_point.safe_sub(start_point)?;
                let total_unlock_duration = self.airdrop_bonus.vesting_duration;

                let amount = ((duration_into_unlock as u128).safe_mul(max_bonus as u128)?)
                    .safe_div(total_unlock_duration as u128)? as u64;
                Ok(amount)
            }
        } else {
            Ok(0)
        }
    }
}

impl PinoMerkleDistributor {
    pub fn get_activation_handler(&self) -> Result<PinoActivationHandler> {
        let activation_type = ActivationType::try_from(self.activation_type).unwrap();
        let curr_point = match activation_type {
            ActivationType::Slot => Clock::get()?.slot,
            ActivationType::Timestamp => Clock::get()?.unix_timestamp as u64,
        };
        Ok(PinoActivationHandler {
            curr_point,
            activation_point: self.activation_point,
            airdrop_bonus: self.airdrop_bonus,
        })
    }

    pub fn accumulate_bonus(&mut self, bonus: u64) -> Result<()> {
        self.airdrop_bonus.total_claimed_bonus =
            self.airdrop_bonus.total_claimed_bonus.safe_add(bonus)?;
        Ok(())
    }

    fn get_max_bonus_for_a_claimant(&self, unlocked_amount: u64) -> Result<u64> {
        let max_total_claim_without_bonus =
            self.max_total_claim.safe_sub(self.airdrop_bonus.total_bonus)? as u128;

        let amount = (unlocked_amount as u128)
            .safe_mul(self.airdrop_bonus.total_bonus as u128)?
            .safe_div(max_total_claim_without_bonus)? as u64;
        Ok(amount)
    }

    pub fn get_bonus_for_a_claimaint(
        &self,
        unlocked_amount: u64,
        activation_handler: &PinoActivationHandler,
    ) -> Result<u64> {
        let max_bonus = self.get_max_bonus_for_a_claimant(unlocked_amount)?;
        activation_handler.get_bonus_for_a_claimaint(max_bonus)
    }
}

/// ---------------------------------------------------------------------------
/// Zero-copy accessors & initializer
/// ---------------------------------------------------------------------------
#[inline(always)]
pub fn md_load<'a>(ai: &'a AccountInfo) -> Result<&'a PinoMerkleDistributor> {
    let data = ai.try_borrow_data()?;
    let body = body_ro(&data, &MERKLE_DISTRIBUTOR_DISC)?;
    if body.len() < PinoMerkleDistributor::BODY_LEN {
        return Err(ProgramError::InvalidAccountData.into());
    }
    // SAFETY: repr(C) + POD; bounds checked above
    Ok(unsafe { &*(body.as_ptr() as *const PinoMerkleDistributor) })
}

#[inline(always)]
pub fn md_load_mut<'a>(ai: &'a AccountInfo) -> Result<&'a mut PinoMerkleDistributor> {
    let mut data = ai.try_borrow_mut_data()?;
    let body = body_rw(&mut data, &MERKLE_DISTRIBUTOR_DISC)?;
    if body.len() < PinoMerkleDistributor::BODY_LEN {
        return Err(ProgramError::InvalidAccountData.into());
    }
    Ok(unsafe { &mut *(body.as_mut_ptr() as *mut PinoMerkleDistributor) })
}

/// Initialize a newly created account's data area:
/// writes the discriminator and zeroes the body.
#[inline(always)]
pub fn md_init_zero(ai: &AccountInfo) -> Result<()> {
    let mut data = ai.try_borrow_mut_data()?;
    if data.len() < 8 {
        return Err(ProgramError::InvalidAccountData.into());
    }
    data[..8].copy_from_slice(&MERKLE_DISTRIBUTOR_DISC);

    let body_len = PinoMerkleDistributor::BODY_LEN;
    if data.len() < 8 + body_len {
        return Err(ProgramError::InvalidAccountData.into());
    }
    let body = &mut data[8..8 + body_len];
    for b in body.iter_mut() { *b = 0; }
    Ok(())
}
