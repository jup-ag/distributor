use anchor_lang::prelude::*;
use solana_program::hash::hashv;

#[account]
#[derive(Default)]
pub struct CanopyTree {
    /// The 256-bit merkle root.
    pub root: [u8; 32],
    /// The depth of merkle will store onchain
    /// With `depth``: total levels from the root to leaves: depth + 1
    pub depth: u8,
    /// A vector of node hashes representing canopy leaves node
    pub nodes: Vec<[u8; 32]>,
    /// The distributor associated with this Merkle tree
    pub distributor: Pubkey,
    /// Buffer
    pub buffer: [u64; 5],
}

impl CanopyTree {
    /// Maximum nodes at level i: 2^i
    /// Example: depth = 2 => maximum canopy nodes = 2^2.
    pub fn space(depth: usize) -> usize {
        let max_nodes = 1 << depth;
        let size = (8 + 33 + 4 + 32 * max_nodes + 32 + 8 * 5) as usize;
        size
    }

    pub fn verify_canopy_root(&self, root: [u8; 32], canopy_nodes: Vec<[u8; 32]>) -> bool {
        let mut current_nodes = canopy_nodes;

        while current_nodes.len() > 1 {
            let mut next_level: Vec<[u8; 32]> = vec![];
            let mut i = 0;
            while i < current_nodes.len() {
                if i + 1 < current_nodes.len() {
                    let lsib = current_nodes[i];
                    let rsib = current_nodes[i + 1];
                    let hashed_node;
                    if lsib <= rsib {
                        hashed_node = hashv(&[&[1u8], &lsib, &rsib]).to_bytes();
                    } else {
                        hashed_node = hashv(&[&[1u8], &rsib, &lsib]).to_bytes();
                    };
                    next_level.push(hashed_node);
                    i += 2;
                } else {
                    next_level.push(current_nodes[i]);
                    i += 1;
                }
            }
            current_nodes = next_level;
        }

        current_nodes[0] == root
    }

    pub fn verify_leaf(&self, proof: Vec<[u8; 32]>, leaf: [u8; 32], leaf_index: u32) -> bool {
        let mut current_hash = leaf;
        let mut current_index = leaf_index;
        for proof_element in proof.into_iter() {
            if current_hash <= proof_element {
                current_hash = hashv(&[&[1u8], &current_hash, &proof_element]).to_bytes();
            } else {
                current_hash = hashv(&[&[1u8], &proof_element, &current_hash]).to_bytes();
            }

            current_index = current_index.checked_div(2).unwrap();
        }

        // Compare current_hash vs canopy nodes
        let expected_hash = self.nodes[current_index as usize];

        current_hash == expected_hash
    }
}

#[cfg(test)]
mod canopy_test {
    use super::*;
    const ROOT: [u8; 32] = [
        149, 44, 101, 93, 7, 225, 23, 191, 86, 53, 145, 35, 119, 19, 172, 27, 31, 77, 106, 39, 46,
        55, 2, 145, 190, 58, 158, 147, 10, 28, 75, 191,
    ];

    #[test]
    fn test_verify_canopy_nodes_case_layer1() {
        let nodes: Vec<[u8; 32]> = Vec::from([
            [
                98, 122, 112, 160, 154, 57, 123, 38, 138, 237, 90, 29, 104, 179, 248, 208, 177,
                206, 20, 24, 18, 197, 73, 220, 182, 102, 240, 180, 223, 71, 3, 113,
            ],
            [
                170, 65, 27, 221, 57, 223, 88, 94, 203, 241, 104, 246, 54, 119, 188, 225, 16, 65,
                109, 98, 73, 121, 49, 218, 50, 213, 136, 253, 205, 65, 213, 225,
            ],
        ]);

        let canopy_nodes = CanopyTree::default();
        let result = canopy_nodes.verify_canopy_root(ROOT, nodes);
        assert_eq!(result, true);
    }

    #[test]
    fn test_verify_canopy_nodes_case_layer2() {
        let nodes: Vec<[u8; 32]> = Vec::from([
            [
                144, 158, 139, 112, 26, 179, 41, 202, 82, 243, 65, 200, 218, 132, 105, 194, 85, 30,
                7, 183, 154, 178, 33, 139, 166, 206, 248, 31, 11, 210, 164, 215,
            ],
            [
                230, 163, 113, 64, 107, 48, 40, 9, 189, 18, 77, 16, 53, 248, 247, 199, 178, 127,
                181, 108, 223, 253, 137, 55, 234, 87, 205, 210, 112, 189, 107, 151,
            ],
            [
                170, 65, 27, 221, 57, 223, 88, 94, 203, 241, 104, 246, 54, 119, 188, 225, 16, 65,
                109, 98, 73, 121, 49, 218, 50, 213, 136, 253, 205, 65, 213, 225,
            ],
        ]);

        let canopy_nodes = CanopyTree::default();
        let result = canopy_nodes.verify_canopy_root(ROOT, nodes);
        assert_eq!(result, true);
    }

    #[test]
    fn test_verify_canopy_nodes_case_layer3() {
        let nodes: Vec<[u8; 32]> = Vec::from([
            [
                144, 158, 139, 112, 26, 179, 41, 202, 82, 243, 65, 200, 218, 132, 105, 194, 85, 30,
                7, 183, 154, 178, 33, 139, 166, 206, 248, 31, 11, 210, 164, 215,
            ],
            [
                230, 163, 113, 64, 107, 48, 40, 9, 189, 18, 77, 16, 53, 248, 247, 199, 178, 127,
                181, 108, 223, 253, 137, 55, 234, 87, 205, 210, 112, 189, 107, 151,
            ],
            [
                170, 65, 27, 221, 57, 223, 88, 94, 203, 241, 104, 246, 54, 119, 188, 225, 16, 65,
                109, 98, 73, 121, 49, 218, 50, 213, 136, 253, 205, 65, 213, 225,
            ],
        ]);

        let canopy_nodes = CanopyTree::default();
        let result = canopy_nodes.verify_canopy_root(ROOT, nodes);
        assert_eq!(result, true);
    }

    #[test]
    fn test_verify_canopy_nodes_case_layer4() {
        let nodes: Vec<[u8; 32]> = Vec::from([
            [
                33, 157, 45, 11, 26, 60, 103, 187, 252, 243, 171, 223, 86, 120, 181, 97, 192, 132,
                46, 183, 5, 136, 21, 227, 189, 227, 15, 100, 87, 60, 126, 125,
            ],
            [
                41, 94, 138, 1, 107, 84, 242, 150, 6, 169, 100, 252, 138, 82, 36, 24, 252, 84, 228,
                124, 237, 219, 68, 212, 215, 111, 253, 141, 173, 45, 35, 229,
            ],
            [
                51, 147, 193, 24, 169, 158, 140, 118, 17, 108, 141, 221, 212, 82, 234, 88, 4, 64,
                109, 209, 148, 214, 84, 78, 184, 8, 130, 210, 172, 169, 198, 151,
            ],
            [
                91, 133, 169, 74, 229, 46, 159, 235, 129, 154, 97, 186, 225, 169, 62, 105, 125, 67,
                158, 69, 168, 115, 109, 254, 205, 237, 71, 42, 15, 240, 64, 192,
            ],
            [
                170, 65, 27, 221, 57, 223, 88, 94, 203, 241, 104, 246, 54, 119, 188, 225, 16, 65,
                109, 98, 73, 121, 49, 218, 50, 213, 136, 253, 205, 65, 213, 225,
            ],
        ]);

        let canopy_nodes = CanopyTree::default();
        let result = canopy_nodes.verify_canopy_root(ROOT, nodes);
        assert_eq!(result, true);
    }
}
