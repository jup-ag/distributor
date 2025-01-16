use crate::error::ErrorCode;
use crate::math::safe_math::SafeMath;
use anchor_lang::{
    account,
    prelude::{Pubkey, *},
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert;

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the activation
pub enum ActivationType {
    Slot,
    Timestamp,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the activation
pub enum ClaimType {
    Permissionless,            // 0
    Permissioned,              // 1, require double signing
    PermissionlessWithStaking, // 2, claim and staking
    PermissionedWithStaking,   // 3, require double signing
}

/// State for the account which distributes tokens.
#[account(zero_copy)]
#[derive(Default, Debug, InitSpace)]
pub struct MerkleDistributor {
    /// The 256-bit merkle root.
    pub root: [u8; 32],
    /// [Mint] of the token to be distributed.
    pub mint: Pubkey,
    /// base key of distributor.
    pub base: Pubkey,
    /// Token Address of the vault
    pub token_vault: Pubkey,
    /// Clawback receiver
    pub clawback_receiver: Pubkey,
    /// Admin wallet
    pub admin: Pubkey,
    /// locker, for claim type claim and stake
    pub locker: Pubkey,
    /// operator for signing in permissioned merkle tree
    pub operator: Pubkey,
    /// Version of the airdrop
    pub version: u64,
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
    /// this merkle tree is activated from this slot or timestamp
    pub activation_point: u64,
    /// activation type, 0 means slot, 1 means timestamp
    pub activation_type: u8,
    /// claim type
    pub claim_type: u8,
    /// Bump seed.
    pub bump: u8,
    /// Whether or not the distributor has been clawed back
    pub clawed_back: u8,
    /// indicate that whether admin can close this pool, for testing purpose
    pub closable: u8,
    /// Padding 0
    pub padding_0: [u8; 3],
    // bonus multiplier
    pub airdrop_bonus: AirdropBonus,
    // padding 2
    pub padding_2: [u128; 5],
}

#[zero_copy]
#[derive(Debug, Default, InitSpace)]
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
    pub curr_point: u64,
    /// activation slot or activation timestamp
    pub activation_point: u64,
    /// bonus multiplier
    pub airdrop_bonus: AirdropBonus,
}

impl ActivationHandler {
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

impl MerkleDistributor {
    pub fn get_activation_handler(&self) -> Result<ActivationHandler> {
        let activation_type = ActivationType::try_from(self.activation_type).unwrap();
        let curr_point = match activation_type {
            ActivationType::Slot => Clock::get()?.slot,
            ActivationType::Timestamp => Clock::get()?.unix_timestamp as u64,
        };
        Ok(ActivationHandler {
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
    pub fn clawed_back(&self) -> bool {
        self.clawed_back == 1
    }
    pub fn set_clawed_back(&mut self) {
        self.clawed_back = 1;
    }

    pub fn closable(&self) -> bool {
        self.closable == 1
    }

    pub fn authorize_claim<'info>(&self, operator: &Option<Signer<'info>>) -> Result<()> {
        // check operator
        let claim_type =
            ClaimType::try_from(self.claim_type).map_err(|_| ErrorCode::TypeCastedError)?;

        require!(
            claim_type == ClaimType::Permissionless || claim_type == ClaimType::Permissioned,
            ErrorCode::InvalidClaimType
        );

        if claim_type == ClaimType::Permissioned {
            // validate operator
            let operator = operator.clone().unwrap();
            require!(operator.key() == self.operator, ErrorCode::InvalidOperator);
        }
        Ok(())
    }

    pub fn authorize_claim_and_stake<'info>(&self, operator: &Option<Signer<'info>>) -> Result<()> {
        // check operator
        let claim_type =
            ClaimType::try_from(self.claim_type).map_err(|_| ErrorCode::TypeCastedError)?;

        require!(
            claim_type == ClaimType::PermissionlessWithStaking
                || claim_type == ClaimType::PermissionedWithStaking,
            ErrorCode::InvalidClaimType
        );

        if claim_type == ClaimType::PermissionedWithStaking {
            // validate operator
            let operator = operator.clone().unwrap();
            require!(operator.key() == self.operator, ErrorCode::InvalidOperator);
        }
        Ok(())
    }

    pub fn signer(&self) -> MerkleDistributorSigner {
        MerkleDistributorSigner {
            base: self.base.to_bytes(),
            mint: self.mint.to_bytes(),
            version: self.version.to_le_bytes(),
            bump: [self.bump],
        }
    }
}

const_assert!(MerkleDistributor::INIT_SPACE == 440);

pub struct MerkleDistributorSigner {
    base: [u8; 32],
    mint: [u8; 32],
    version: [u8; 8],
    bump: [u8; 1],
}

impl MerkleDistributorSigner {
    pub fn seeds(&self) -> [&[u8]; 5] {
        [
            b"MerkleDistributor".as_ref(),
            &self.base,
            &self.mint,
            &self.version,
            &self.bump,
        ]
    }
}

// #[test]
// fn test_size() {
//     println!("{} ", MerkleDistributor::INIT_SPACE)
// }
