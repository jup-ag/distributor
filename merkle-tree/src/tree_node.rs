use std::str::FromStr;

use crate::csv_entry::CsvEntry;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{Deserialize, Serialize};
use solana_program::{hash::hashv, pubkey::Pubkey};
use solana_sdk::hash::Hash;

/// Represents the claim information for an account.
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    /// Unique index in the merkle tree (bitmap bit position)
    pub index: u32,

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
    /// Leaf = hash( LEAF_PREFIX || hash( claimant || index_be || amount_be || locked_be ) )
    /// Make sure this exactly matches handle_new_claim.
    pub fn hash(&self) -> Hash {
     hashv(&[
            &self.claimant.to_bytes(),
            &self.index.to_be_bytes(),
            &self.amount.to_be_bytes(),
            &self.locked_amount.to_be_bytes(),
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

    /// Convenience: set index after CSV load
    pub fn with_index(mut self, index: u32) -> Self {
        self.index = index;
        self
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
    pub fn from_csv(entry: CsvEntry, decimals: u32, index: u32) -> Self {
        Self {
            index, // can fill later with enumerate() or with_index()
            claimant: Pubkey::from_str(entry.pubkey.as_str()).unwrap(),
            amount: ui_amount_to_token_amount(entry.amount.as_str(), decimals),
            locked_amount: ui_amount_to_token_amount(entry.locked_amount.as_str(), decimals),
            proof: None,
        }
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
            .enumerate()
            .map(|(i, x)| TreeNode::from_csv(x, decimals, i as u32))
            .collect();

        assert_eq!(tree_nodes[0].amount, 1000123456);
        assert_eq!(tree_nodes[0].locked_amount, 9123456);
        assert_eq!(tree_nodes[1].amount, 2000123456);
        assert_eq!(tree_nodes[1].locked_amount, 8123456);
        assert_eq!(tree_nodes[2].amount, 1500123456);
        assert_eq!(tree_nodes[2].locked_amount, 7123456);

        // sanity: hash should be stable and depend on index
        let h0 = tree_nodes[0].hash();
        let mut node_with_different_index = tree_nodes[0].clone();
        node_with_different_index.index = 1;
        let h1 = node_with_different_index.hash();
        assert_ne!(h0, h1);
    }
}
