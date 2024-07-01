use crate::math::safe_math::SafeMath;
use anchor_lang::{
    account,
    prelude::{Pubkey, *},
};
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
    /// this merkle tree is enable from this slot
    pub enable_slot: u64,
    /// indicate that whether admin can close this pool, for testing purpose
    pub closable: bool,
    /// bonus multiplier
    pub airdrop_bonus: AirdropBonus,
    /// min_locked_duration
    pub min_locked_duration: u64,
    /// locker
    pub locker: Pubkey,
    /// Buffer 2
    pub buffer_2: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct AirdropBonus {
    /// total bonus
    pub total_bonus: u64,
    // vesting slot duration
    pub vesting_slot_duration: u64,
    /// total bonus
    pub total_claimed_bonus: u64,
}

impl MerkleDistributor {
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
    pub fn get_bonus_for_a_claimaint(&self, unlocked_amount: u64, curr_slot: u64) -> Result<u64> {
        let max_bonus = self.get_max_bonus_for_a_claimant(unlocked_amount)?;

        let start_slot = self.enable_slot;
        let end_slot = self
            .airdrop_bonus
            .vesting_slot_duration
            .safe_add(start_slot)?;

        if curr_slot >= start_slot {
            if curr_slot >= end_slot {
                Ok(max_bonus)
            } else {
                let slot_into_unlock = curr_slot.safe_sub(start_slot)?;
                let total_unlock_slot = self.airdrop_bonus.vesting_slot_duration;

                let amount = ((slot_into_unlock as u128).safe_mul(max_bonus as u128)?)
                    .safe_div(total_unlock_slot as u128)? as u64;
                Ok(amount)
            }
        } else {
            Ok(0)
        }
    }
}

impl MerkleDistributor {
    pub const LEN: usize = 8 + std::mem::size_of::<MerkleDistributor>();
}
