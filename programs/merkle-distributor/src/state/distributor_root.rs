use anchor_lang::prelude::*;
use crate::math::safe_math::SafeMath;
use static_assertions::const_assert;

/// Parent Account: Authority of parent vault use to distribute fund to all distributors
#[account(zero_copy)]
#[derive(Default, InitSpace)]
pub struct DistributorRoot {
    /// Bump seed.
    pub bump: u8,
    /// padding 0
    pub padding_0: [u8; 7],
    /// Mint of the token to be distributed.
    pub mint: Pubkey,
    /// Base key of distributor root
    pub base: Pubkey,
    /// Token Address of distributor root vault
    pub distributor_root_vault: Pubkey,
    /// Max claim amount
    pub max_claim_amount: u64,
    /// Max distributor
    pub max_distributor: u64,
    /// total funded amount
    pub total_funded_amount: u64,
    /// total escrow created
    pub total_distributor_created: u64,
    /// Buffer for future use or alignment.
    pub buffer: [u128; 5],
}

const_assert!(DistributorRoot::INIT_SPACE == 216);

impl DistributorRoot {
    /// Returns the DistributorRootVaultSigner for PDA signing.
    pub fn signer(&self) -> DistributorRootVaultSigner {
        DistributorRootVaultSigner {
            base: self.base.to_bytes(),
            mint: self.mint.to_bytes(),
            bump: [self.bump],
        }
    }

    pub fn get_and_set_fund_amount(&mut self, max_amount: u64) -> Result<u64> {
        let max_amount_should_be_funded =
            self.max_claim_amount.safe_sub(self.total_funded_amount)?;
        let funded_amount = max_amount_should_be_funded.min(max_amount);

        self.total_funded_amount = self.total_funded_amount.safe_add(funded_amount)?;
        Ok(funded_amount)
    }

    pub fn update_new_distributor(&mut self) -> Result<()> {
        self.total_distributor_created = self.total_distributor_created.safe_add(1)?;
        Ok(())
    }
}

/// Representing the signer seeds for the DistributorRootVault.
pub struct DistributorRootVaultSigner {
    base: [u8; 32],
    mint: [u8; 32],
    bump: [u8; 1],
}

impl DistributorRootVaultSigner {
    /// Returns the seeds required for PDA signing.
    pub fn seeds(&self) -> [&[u8]; 4] {
        [
            b"DistributorRoot".as_ref(),
            &self.base,
            &self.mint,
            &self.bump,
        ]
    }
}
