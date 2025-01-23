use anchor_lang::prelude::*;
/// Holds whether or not a claimant has claimed tokens.
#[account]
#[derive(Default)]
pub struct PartialMerkleTree {
    /// The 256-bit merkle root.
    pub root: [u8; 32],
    /// The number of layers in the tree that are stored on-chain, excluding the root layer.
    pub depth: u8,
    /// The total number of nodes stored on-chain. This is used for breadth-first search (BFS) traversal.
    pub total_nodes: u8,
    /// A vector of node hashes representing a subset of the Merkle tree, excluding the root.
    /// The length of this vector should be equal to `total_nodes`.
    pub nodes: Vec<[u8; 32]>,
    /// The distributor associated with this Merkle tree
    pub distributor: Pubkey,
    /// Buffer
    pub buffer: [u64; 5],
}

impl PartialMerkleTree {
    /// Total nodes: 2^(depth + 1) - 2
    pub fn space(total_nodes: usize) -> usize {
        let size = (8 + 33 + 4 + 32 * total_nodes + 32 + 8 * 5) as usize;
        size
    }
}
