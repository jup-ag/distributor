use crate::error::ErrorCode;
use crate::math::safe_math::SafeMath;
use anchor_lang::{
    account,
    prelude::{Pubkey, *},
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the activation
pub enum ActivationType {
    Slot,
    Timestamp,
}

/// State for the account which distributes tokens.
#[account]
#[derive(Default, Debug)]
pub struct MerkleDistributor {
    /// Bump seed.
    pub bump: u8,
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
    /// Maximum number of tokens that can ever be claimed from this [MerkleDistributor].
    pub max_total_claim: u64,
    /// Maximum number of nodes in [MerkleDistributor].
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
    /// Whether or not the distributor has been clawed back
    pub clawed_back: bool,
    /// this merkle tree is activated from this slot
    pub activation_slot: u64,
    /// indicate that whether admin can close this pool, for testing purpose
    pub closable: bool,
    /// bonus multiplier
    pub airdrop_bonus: AirdropBonus,
    /// activation timstamp
    pub activation_timestamp: u64,
    /// activation type, 0 means slot, 1 means timestamp
    pub activation_type: u8,
    /// Buffer 1
    pub buffer_1: [u8; 31],
    /// Buffer 2
    pub buffer_2: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct AirdropBonus {
    /// total bonus
    pub total_bonus: u64,
    // vesting duration
    pub vesting_duration: u64,
    /// total bonus
    pub total_claimed_bonus: u64,
}

pub struct ActivationHandler {
    /// current slot or current timestamp
    pub curr_time: u64,
    /// activation slot or activation timestamp
    pub activation_time: u64,
    /// bonus multiplier
    pub airdrop_bonus: AirdropBonus,
}

impl ActivationHandler {
    pub fn validate_claim(&self) -> Result<()> {
        require!(
            self.activation_time <= self.curr_time,
            ErrorCode::ClaimingIsNotStarted
        );
        Ok(())
    }
    pub fn get_bonus_for_a_claimaint(&self, max_bonus: u64) -> Result<u64> {
        let curr_time = self.curr_time;
        let start_time = self.activation_time;
        let end_time = self.airdrop_bonus.vesting_duration.safe_add(start_time)?;

        if curr_time >= start_time {
            if curr_time >= end_time {
                Ok(max_bonus)
            } else {
                let duration_into_unlock = curr_time.safe_sub(start_time)?;
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

impl MerkleDistributor {
    pub fn get_activation_handler(&self) -> Result<ActivationHandler> {
        let activation_type = ActivationType::try_from(self.activation_type).unwrap();
        match activation_type {
            ActivationType::Slot => Ok(ActivationHandler {
                curr_time: Clock::get()?.slot,
                activation_time: self.activation_slot,
                airdrop_bonus: self.airdrop_bonus,
            }),
            ActivationType::Timestamp => Ok(ActivationHandler {
                curr_time: Clock::get()?.unix_timestamp as u64,
                activation_time: self.activation_timestamp,
                airdrop_bonus: self.airdrop_bonus,
            }),
        }
    }
    pub fn accumulate_bonus(&mut self, bonus: u64) -> Result<()> {
        self.airdrop_bonus.total_claimed_bonus =
            self.airdrop_bonus.total_claimed_bonus.safe_add(bonus)?;
        Ok(())
    }
    fn get_max_bonus_for_a_claimant(&self, unlocked_amount: u64) -> Result<u64> {
        let max_total_claim_without_bonus =
            self.max_total_claim
                .safe_sub(self.airdrop_bonus.total_bonus)? as u128;

        let amount = (unlocked_amount as u128)
            .safe_mul(self.airdrop_bonus.total_bonus as u128)?
            .safe_div(max_total_claim_without_bonus)? as u64;
        Ok(amount)
    }
    pub fn get_bonus_for_a_claimaint(
        &self,
        unlocked_amount: u64,
        activation_handler: &ActivationHandler,
    ) -> Result<u64> {
        let max_bonus = self.get_max_bonus_for_a_claimant(unlocked_amount)?;
        activation_handler.get_bonus_for_a_claimaint(max_bonus)
    }
}

impl MerkleDistributor {
    pub const LEN: usize = 8 + std::mem::size_of::<MerkleDistributor>();
}
