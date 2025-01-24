use solana_program::hash::hashv;

/// modified version of https://github.com/saber-hq/merkle-distributor/blob/ac937d1901033ecb7fa3b0db22f7b39569c8e052/programs/merkle-distributor/src/merkle_proof.rs#L8
/// This function deals with verification of Merkle trees (hash trees).
/// Direct port of https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v3.4.0/contracts/cryptography/MerkleProof.sol
/// Returns true if a `leaf` can be proved to be a part of a Merkle tree
/// defined by `root`. For this, a `proof` must be provided, containing
/// sibling hashes on the branch from the leaf to the root of the tree. Each
/// pair of leaves and each pair of pre-images are assumed to be sorted.
pub fn verify(proof: Vec<[u8; 32]>, root: [u8; 32], leaf: [u8; 32]) -> bool {
    let mut computed_hash = leaf;
    for proof_element in proof.into_iter() {
        if computed_hash <= proof_element {
            // Hash(current computed hash + current element of the proof)
            computed_hash = hashv(&[&[1u8], &computed_hash, &proof_element]).to_bytes();
        } else {
            // Hash(current element of the proof + current computed hash)
            computed_hash = hashv(&[&[1u8], &proof_element, &computed_hash]).to_bytes();
        }
    }
    // Check if the computed hash (root) is equal to the provided root
    computed_hash == root
}

pub fn bfs_index(level: usize, i: usize) -> usize {
    // 2^level - 2 + i
    // Level 1: 2^1 - 2 = 0
    // Level 2: (2^2) - 2 = 2
    // Level 3: (2^3) - 2 = 6
    (1 << level) - 2 + i
}

pub fn verify_partial_merkle(
    root: [u8; 32],
    leaf: [u8; 32],
    depth: u8,
    leaf_index: u8,
    proof: Vec<[u8; 32]>,
    nodes: Vec<[u8; 32]>,
) -> bool {
    let mut current_hash = leaf;
    let mut index = leaf_index;
    for proof_element in proof.clone().into_iter() {
        if current_hash <= proof_element {
            current_hash = hashv(&[&[1u8], &current_hash, &proof_element]).to_bytes();
        } else {
            current_hash = hashv(&[&[1u8], &proof_element, &current_hash]).to_bytes();
        }

        index = index.checked_div(2).unwrap();
    }

    if proof.len() == 0 {
        index = index.checked_div(2).unwrap()
    }

    // Compare current_hash vs partial_nodes[BFS_index(level, index)]
    let expected_hash = nodes[bfs_index(depth as usize, index as usize)];
    if current_hash != expected_hash {
        return false;
    }

    // From sliced tree to root
    let mut curr_hash = current_hash;
    let mut curr_level = depth;
    let mut curr_index = index;
    if proof.len() == 0 {
        curr_index = curr_index.checked_div(2).unwrap();
        curr_level = curr_level.checked_sub(1).unwrap();
    }

    while curr_level > 0 {
        // Find sibling
        let sibling_index = curr_index ^ 1;
        let mut sibling_bfs_idx = bfs_index(curr_level as usize, sibling_index as usize);
        if sibling_bfs_idx as usize >= nodes.len() {
            sibling_bfs_idx = sibling_bfs_idx.checked_sub(1).unwrap();
        }
        let sibling_hash = nodes[sibling_bfs_idx];

        if curr_hash <= sibling_hash {
            curr_hash = hashv(&[&[1u8], &curr_hash, &sibling_hash]).to_bytes();
        } else {
            curr_hash = hashv(&[&[1u8], &sibling_hash, &curr_hash]).to_bytes();
        }

        curr_level = curr_level.checked_sub(1).unwrap();
        curr_index = curr_index.checked_div(2).unwrap();
    }
    curr_hash == root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_partial_merkle_without_proof() {
        let nodes: Vec<[u8; 32]> = vec![
            [
                126, 64, 122, 133, 166, 245, 206, 172, 165, 62, 184, 187, 253, 195, 131, 108, 17,
                173, 142, 132, 212, 106, 14, 59, 227, 95, 146, 125, 110, 192, 94, 55,
            ],
            [
                234, 247, 3, 255, 201, 48, 119, 136, 153, 30, 11, 123, 82, 7, 135, 108, 88, 175, 0,
                101, 239, 68, 69, 27, 24, 23, 119, 76, 236, 185, 111, 165,
            ],
            [
                249, 115, 79, 222, 70, 219, 166, 6, 4, 74, 120, 93, 236, 54, 221, 138, 0, 70, 9,
                202, 196, 24, 232, 157, 202, 8, 35, 230, 202, 135, 49, 197,
            ],
            [
                175, 124, 75, 209, 14, 141, 11, 254, 209, 37, 145, 196, 146, 164, 73, 56, 63, 168,
                205, 36, 21, 32, 227, 85, 210, 14, 105, 98, 85, 1, 215, 239,
            ],
            [
                234, 247, 3, 255, 201, 48, 119, 136, 153, 30, 11, 123, 82, 7, 135, 108, 88, 175, 0,
                101, 239, 68, 69, 27, 24, 23, 119, 76, 236, 185, 111, 165,
            ],
        ];

        let leaf: [u8; 32] = [
            234, 247, 3, 255, 201, 48, 119, 136, 153, 30, 11, 123, 82, 7, 135, 108, 88, 175, 0,
            101, 239, 68, 69, 27, 24, 23, 119, 76, 236, 185, 111, 165,
        ];

        let proof: Vec<[u8; 32]> = vec![];

        let root: [u8; 32] = [
            253, 24, 27, 203, 106, 197, 56, 217, 94, 211, 101, 205, 85, 255, 184, 44, 247, 197,
            108, 228, 180, 216, 70, 129, 67, 162, 219, 185, 178, 198, 131, 72,
        ];
        let index = 4;
        let res = verify_partial_merkle(root, leaf, 2, index, proof, nodes);
        assert!(res)
    }

    #[test]
    fn test_verify_partial_merkle() {
        let nodes: Vec<[u8; 32]> = vec![
            [
                157, 203, 83, 31, 9, 208, 192, 3, 201, 46, 165, 112, 246, 35, 115, 158, 64, 177,
                185, 59, 108, 100, 156, 165, 126, 118, 255, 158, 0, 54, 53, 175,
            ],
            [
                87, 71, 116, 230, 61, 194, 89, 51, 115, 255, 97, 207, 194, 9, 176, 247, 144, 236,
                241, 216, 18, 229, 91, 157, 149, 175, 15, 122, 103, 147, 85, 209,
            ],
            [
                62, 216, 170, 39, 82, 105, 212, 99, 194, 62, 63, 31, 183, 241, 117, 25, 46, 128,
                236, 167, 208, 249, 219, 74, 236, 15, 203, 169, 71, 35, 101, 246,
            ],
            [
                83, 152, 228, 31, 141, 102, 72, 192, 118, 247, 159, 138, 67, 173, 157, 1, 87, 163,
                65, 111, 95, 50, 123, 250, 111, 111, 254, 185, 230, 201, 104, 37,
            ],
            [
                185, 207, 249, 117, 66, 169, 158, 167, 27, 51, 37, 239, 242, 192, 182, 22, 250,
                204, 38, 24, 72, 82, 112, 155, 18, 106, 187, 109, 10, 9, 195, 212,
            ],
            [
                9, 114, 24, 172, 111, 156, 91, 21, 10, 214, 95, 44, 91, 213, 30, 91, 238, 165, 137,
                215, 239, 34, 52, 0, 251, 43, 130, 94, 188, 105, 90, 133,
            ],
        ];

        let leaf: [u8; 32] = [
            72, 15, 168, 106, 121, 139, 104, 195, 0, 91, 56, 197, 115, 84, 190, 201, 167, 38, 128,
            236, 251, 154, 135, 128, 21, 205, 205, 18, 83, 34, 60, 250,
        ];

        let proof: Vec<[u8; 32]> = vec![
            [
                51, 227, 52, 44, 225, 103, 118, 164, 148, 223, 148, 76, 197, 239, 0, 84, 100, 83,
                176, 161, 97, 222, 183, 55, 178, 186, 211, 243, 167, 38, 126, 64,
            ],
            [
                232, 191, 145, 132, 156, 200, 103, 194, 223, 182, 71, 34, 63, 135, 185, 30, 6, 233,
                247, 199, 181, 39, 95, 98, 100, 135, 106, 58, 91, 97, 130, 88,
            ],
        ];

        let root: [u8; 32] = [
            26, 152, 1, 201, 215, 171, 246, 198, 156, 83, 34, 234, 99, 132, 94, 201, 173, 6, 251,
            152, 165, 209, 108, 102, 204, 226, 190, 234, 164, 74, 66, 51,
        ];
        let index = 3;
        let res = verify_partial_merkle(root, leaf, 2, index, proof, nodes);
        assert!(res);
    }
}
