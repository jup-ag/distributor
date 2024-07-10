use solana_program::pubkey::Pubkey;

use crate::{merkle_tree::MerkleTree, tree_node::TreeNode};

pub fn get_proof(merkle_tree: &MerkleTree, index: usize) -> Vec<[u8; 32]> {
    let mut proof = Vec::new();
    let path = merkle_tree.find_path(index).expect("path to index");
    for branch in path.get_proof_entries() {
        if let Some(hash) = branch.get_left_sibling() {
            proof.push(hash.to_bytes());
        } else if let Some(hash) = branch.get_right_sibling() {
            proof.push(hash.to_bytes());
        } else {
            panic!("expected some hash at each level of the tree");
        }
    }
    proof
}

/// Given a set of tree nodes, get the total unlocked amount. Panics on overflow
pub fn get_total_unlocked_amount(nodes: &[TreeNode]) -> u64 {
    nodes
        .iter()
        .try_fold(0, |acc: u64, n| acc.checked_add(n.amount))
        .unwrap()
}

pub fn get_total_locked_amount(nodes: &[TreeNode]) -> u64 {
    nodes
        .iter()
        .try_fold(0, |acc: u64, n| acc.checked_add(n.locked_amount))
        .unwrap()
}

pub fn get_merkle_distributor_pda(
    program_id: &Pubkey,
    base: &Pubkey,
    mint: &Pubkey,
    version: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"MerkleDistributor".as_ref(),
            base.as_ref(),
            mint.as_ref(),
            version.to_le_bytes().as_ref(),
        ],
        program_id,
    )
}

pub fn get_claim_status_pda(
    program_id: &Pubkey,
    claimant: &Pubkey,
    distributor: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"ClaimStatus".as_ref(),
            claimant.to_bytes().as_ref(),
            distributor.to_bytes().as_ref(),
        ],
        program_id,
    )
}

#[derive(Debug)]
pub struct MerkleValidationError {
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function to create a tree node
    fn create_node(claimant: Pubkey, amount: u64, locked_amount: u64) -> TreeNode {
        TreeNode {
            claimant,
            amount,
            locked_amount,
            proof: None,
        }
    }

    #[test]
    fn test_get_max_total_claim_no_overflow() {
        let nodes = vec![
            create_node(Pubkey::new_unique(), 100, 10),
            create_node(Pubkey::new_unique(), 300, 20),
        ];

        let total_unlocked_amount = get_total_unlocked_amount(&nodes);
        assert_eq!(total_unlocked_amount, 400); // 100 + 300

        let total_locked_amount = get_total_locked_amount(&nodes);
        assert_eq!(total_locked_amount, 30); // 10 + 20
    }
}
