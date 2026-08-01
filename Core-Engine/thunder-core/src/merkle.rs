// ---------------------------------------------------------------------------
//  Thunder Blockchain — Merkle Tree
// ---------------------------------------------------------------------------
//  Binary Merkle tree for transaction integrity verification.
//  Provides root computation, proof generation, and proof verification.
// ---------------------------------------------------------------------------

use crate::crypto::{self, Hash};

/// Compute the Merkle root of a list of transaction hashes.
///
/// If the list has an odd number of elements the last element is duplicated.
/// An empty list returns a zero hash.
pub fn compute_merkle_root(hashes: &[Hash]) -> Hash {
    if hashes.is_empty() {
        return [0u8; 32];
    }
    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut current_level: Vec<Hash> = hashes.to_vec();

    while current_level.len() > 1 {
        // Duplicate last element if the level has an odd count.
        if current_level.len() % 2 != 0 {
            let last = *current_level.last().unwrap();
            current_level.push(last);
        }

        let mut next_level = Vec::with_capacity(current_level.len() / 2);
        for pair in current_level.chunks(2) {
            next_level.push(hash_pair(&pair[0], &pair[1]));
        }
        current_level = next_level;
    }

    current_level[0]
}

/// A single node in a Merkle proof.
#[derive(Debug, Clone)]
pub struct MerkleProofNode {
    pub hash: Hash,
    pub is_left: bool,
}

/// Generate a Merkle proof for the element at `index` within `hashes`.
pub fn generate_proof(hashes: &[Hash], index: usize) -> Vec<MerkleProofNode> {
    if hashes.is_empty() || index >= hashes.len() {
        return Vec::new();
    }

    let mut proof = Vec::new();
    let mut current_level: Vec<Hash> = hashes.to_vec();
    let mut idx = index;

    while current_level.len() > 1 {
        if current_level.len() % 2 != 0 {
            let last = *current_level.last().unwrap();
            current_level.push(last);
        }

        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        proof.push(MerkleProofNode {
            hash: current_level[sibling_idx],
            is_left: idx % 2 != 0,
        });

        // Move to the next level.
        let mut next_level = Vec::with_capacity(current_level.len() / 2);
        for pair in current_level.chunks(2) {
            next_level.push(hash_pair(&pair[0], &pair[1]));
        }
        current_level = next_level;
        idx /= 2;
    }

    proof
}

/// Verify that `leaf_hash` is part of the tree with the given `root`.
pub fn verify_proof(root: &Hash, leaf_hash: &Hash, proof: &[MerkleProofNode]) -> bool {
    let mut current = *leaf_hash;
    for node in proof {
        current = if node.is_left {
            hash_pair(&node.hash, &current)
        } else {
            hash_pair(&current, &node.hash)
        };
    }
    current == *root
}

// ── Internal ───────────────────────────────────────────────────────────────

/// Hash two child nodes together to form a parent node.
fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    let mut combined = Vec::with_capacity(64);
    combined.extend_from_slice(left);
    combined.extend_from_slice(right);
    crypto::hash_sha256(&combined)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash_sha256;

    #[test]
    fn test_empty_tree() {
        assert_eq!(compute_merkle_root(&[]), [0u8; 32]);
    }

    #[test]
    fn test_single_element() {
        let h = hash_sha256(b"tx1");
        assert_eq!(compute_merkle_root(&[h]), h);
    }

    #[test]
    fn test_two_elements() {
        let h1 = hash_sha256(b"tx1");
        let h2 = hash_sha256(b"tx2");
        let root = compute_merkle_root(&[h1, h2]);
        assert_ne!(root, h1);
        assert_ne!(root, h2);
    }

    #[test]
    fn test_deterministic() {
        let hashes: Vec<Hash> = (0..5).map(|i| hash_sha256(&[i as u8])).collect();
        let r1 = compute_merkle_root(&hashes);
        let r2 = compute_merkle_root(&hashes);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_proof_roundtrip() {
        let hashes: Vec<Hash> = (0..8).map(|i| hash_sha256(&[i as u8])).collect();
        let root = compute_merkle_root(&hashes);

        for i in 0..hashes.len() {
            let proof = generate_proof(&hashes, i);
            assert!(verify_proof(&root, &hashes[i], &proof));
        }
    }

    #[test]
    fn test_proof_invalid_leaf() {
        let hashes: Vec<Hash> = (0..4).map(|i| hash_sha256(&[i as u8])).collect();
        let root = compute_merkle_root(&hashes);
        let proof = generate_proof(&hashes, 0);
        let fake_hash = hash_sha256(b"fake");
        assert!(!verify_proof(&root, &fake_hash, &proof));
    }
}
