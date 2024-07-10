use std::str::FromStr;

use crate::csv_entry::CsvEntry;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{Deserialize, Serialize};
use solana_program::{hash::hashv, pubkey::Pubkey};
use solana_sdk::hash::Hash;

/// Represents the claim information for an account.
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    /// Pubkey of the claimant; will be responsible for signing the claim
    pub claimant: Pubkey,
    /// Amount that claimant can claim
    pub amount: u64,
    /// Locked amount
    pub locked_amount: u64,
    /// Claimant's proof of inclusion in the Merkle Tree
    pub proof: Option<Vec<[u8; 32]>>,
}

impl TreeNode {
    pub fn hash(&self) -> Hash {
        hashv(&[
            &self.claimant.to_bytes(),
            &self.amount.to_le_bytes(),
            &self.locked_amount.to_le_bytes(),
        ])
    }
    /// Return total amount for this claimant
    pub fn total_amount(&self) -> u64 {
        self.amount.checked_add(self.locked_amount).unwrap()
    }

    /// Return unlocked amount for this claimant
    pub fn unlocked_amount(&self) -> u64 {
        self.amount
    }
    /// Return locked amount for this claimant
    pub fn locked_amount(&self) -> u64 {
        self.locked_amount
    }
}

/// Converts a ui amount to a token amount (with decimals)
pub fn ui_amount_to_token_amount(amount: &str, decimals: u32) -> u64 {
    let amount = Decimal::from_str(amount).unwrap();
    let amount = amount
        .checked_mul(Decimal::from_u64(10u64.checked_pow(decimals).unwrap()).unwrap())
        .unwrap();
    let amount = amount.floor();
    amount.try_into().unwrap()
}

impl TreeNode {
    pub fn from_csv(entry: CsvEntry, decimals: u32) -> Self {
        let node = Self {
            claimant: Pubkey::from_str(entry.pubkey.as_str()).unwrap(),
            amount: ui_amount_to_token_amount(entry.amount.as_str(), decimals),
            locked_amount: ui_amount_to_token_amount(entry.locked_amount.as_str(), decimals),
            proof: None,
        };
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn test_ui_amount_to_token_amount() {
        let amount = "3.1234";
        assert_eq!(ui_amount_to_token_amount(amount, 3), 3123);

        let amount = "0.1234";
        assert_eq!(ui_amount_to_token_amount(amount, 2), 12);

        let amount = "0.00001";
        assert_eq!(ui_amount_to_token_amount(amount, 2), 0);
    }

    #[test]
    fn test_csv_decimals_parsing() {
        let path = PathBuf::from("./test_fixtures/test_csv_decimal.csv");
        let entries = CsvEntry::new_from_file(&path).expect("Failed to parse CSV");
        assert_eq!(entries.len(), 3);
        let decimals = 6;

        let tree_nodes: Vec<TreeNode> = entries
            .into_iter()
            .map(|x| TreeNode::from_csv(x, decimals))
            .collect();

        assert_eq!(tree_nodes[0].amount, 1000123456);
        assert_eq!(tree_nodes[0].locked_amount, 9123456);
        assert_eq!(tree_nodes[1].amount, 2000123456);
        assert_eq!(tree_nodes[1].locked_amount, 8123456);
        assert_eq!(tree_nodes[2].amount, 1500123456);
        assert_eq!(tree_nodes[2].locked_amount, 7123456);
    }
}
