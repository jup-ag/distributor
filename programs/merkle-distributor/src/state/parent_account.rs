use anchor_lang::prelude::*;
use static_assertions::const_assert;

/// Parent Account: Authority of parent vault use to distribute fund to all distributors
#[account(zero_copy)]
#[derive(Default, InitSpace)]
pub struct ParentAccount {
    /// Bump seed.
    pub bump: u8,
    /// padding 0
    pub padding_0: [u8; 7],
    /// Admin of merkle tree, store for for testing purpose
    pub admin: Pubkey,
    /// Mint of the token to be distributed.
    pub mint: Pubkey,
    /// Token Address of parent vault
    pub parent_vault: Pubkey,
    /// Padding
    pub padding: u64,
    /// Buffer for future use or alignment.
    pub buffer: [u128; 5]
}

const_assert!(ParentAccount::INIT_SPACE == 192);

impl ParentAccount {

    /// Returns the ParentVaultSigner for PDA signing.
    pub fn signer(&self) -> ParentVaultSigner {
        ParentVaultSigner {
            mint: self.mint.to_bytes(),
            bump: [self.bump],
        }
    }
}

/// Representing the signer seeds for the ParentVault.
pub struct ParentVaultSigner {
    mint: [u8; 32],
    bump: [u8; 1],
}

impl ParentVaultSigner {
    /// Returns the seeds required for PDA signing.
    pub fn seeds(&self) -> [&[u8]; 3] {
        [b"ParentAccount".as_ref(), &self.mint, &self.bump]
    }
}
