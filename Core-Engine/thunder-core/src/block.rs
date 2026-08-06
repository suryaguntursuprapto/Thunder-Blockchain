// ---------------------------------------------------------------------------
//  Thunder Blockchain — Block & Block Header
// ---------------------------------------------------------------------------
//  Defines the `Block` and `BlockHeader` structures.  Each block links to its
//  predecessor via `prev_hash` and contains a Merkle root of its txs.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::crypto::{self, Address, Hash, PublicKey, Signature};
use crate::merkle;
use crate::transaction::Transaction;

// ── Block Header ───────────────────────────────────────────────────────────

/// The header of a block — contains all metadata but not the transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block height (0 = genesis).
    pub height: u64,
    /// Unix timestamp (seconds).
    pub timestamp: u64,
    /// Hash of the previous block.
    pub prev_hash: Hash,
    /// Merkle root of the world-state after applying this block.
    pub state_root: Hash,
    /// Merkle root of the transactions in this block.
    pub tx_root: Hash,
    /// Address of the validator who proposed this block.
    pub validator: Address,
    /// Validator's public key.
    pub validator_pubkey: PublicKey,
    /// Base Fee burned per transaction recursively (EIP-1559 Gwei rules)
    pub base_fee: u64,
    /// Absolute Total Payout received by the Validator (Base Subsidy + Priority Tips)
    pub reward: u64,
    /// Validator's signature over the header hash.
    #[serde(with = "BigArray")]
    pub signature: Signature,
}

impl BlockHeader {
    /// Compute the hash of this header (excluding the signature field).
    pub fn hash(&self) -> Hash {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.height.to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.prev_hash);
        buf.extend_from_slice(&self.state_root);
        buf.extend_from_slice(&self.tx_root);
        buf.extend_from_slice(&self.validator);
        buf.extend_from_slice(&self.validator_pubkey);
        buf.extend_from_slice(&self.base_fee.to_le_bytes());
        buf.extend_from_slice(&self.reward.to_le_bytes());
        crypto::hash_sha256(&buf)
    }
}

// ── Block ──────────────────────────────────────────────────────────────────

/// A complete block: header + ordered list of transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create the genesis block (height 0, no predecessor, no transactions).
    pub fn genesis() -> Self {
        let header = BlockHeader {
            height: 0,
            timestamp: 0,
            prev_hash: [0u8; 32],
            state_root: [0u8; 32],
            tx_root: [0u8; 32],
            validator: [0u8; 20],
            validator_pubkey: [0u8; 32],
            base_fee: 1,                // Genesis minimum 1 Gwei Base Fee
            reward: 50 * 1_000_000_000, // Genesis standard 50 THDR subsidy tracking 9-Decimals natively
            signature: [0u8; 64],
        };
        Self {
            header,
            transactions: Vec::new(),
        }
    }

    /// Create a new block on top of the given previous block.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        prev_block: &Block,
        transactions: Vec<Transaction>,
        state_root: Hash,
        validator: Address,
        validator_pubkey: PublicKey,
        timestamp: u64,
        base_fee: u64,
        reward: u64,
    ) -> Self {
        let tx_hashes: Vec<Hash> = transactions.iter().map(|tx| tx.hash()).collect();
        let tx_root = merkle::compute_merkle_root(&tx_hashes);

        let header = BlockHeader {
            height: prev_block.header.height + 1,
            timestamp,
            prev_hash: prev_block.header.hash(),
            state_root,
            tx_root,
            validator,
            validator_pubkey,
            base_fee,
            reward,
            signature: [0u8; 64], // Placeholder — must be signed afterwards.
        };

        Self {
            header,
            transactions,
        }
    }

    /// Sign this block's header in-place with the validator's key pair.
    pub fn sign(&mut self, key_pair: &crypto::KeyPair) {
        let hash = self.header.hash();
        self.header.signature = key_pair.sign(&hash);
    }

    /// Verify the block's header signature.
    pub fn verify_signature(&self) -> bool {
        let hash = self.header.hash();
        crypto::verify_signature(&self.header.validator_pubkey, &hash, &self.header.signature)
    }

    /// The block hash (same as header hash).
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    /// Verify internal consistency: tx_root matches the transactions.
    pub fn verify_tx_root(&self) -> bool {
        let tx_hashes: Vec<Hash> = self.transactions.iter().map(|tx| tx.hash()).collect();
        let expected = merkle::compute_merkle_root(&tx_hashes);
        expected == self.header.tx_root
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.header.prev_hash, [0u8; 32]);
        assert!(genesis.transactions.is_empty());
    }

    #[test]
    fn test_block_creation_and_signing() {
        let validator = KeyPair::generate();
        let genesis = Block::genesis();

        let mut block = Block::new(
            &genesis,
            Vec::new(),
            [0u8; 32],
            validator.address(),
            validator.public_key(),
            1000,
            1,
            5000000,
        );
        block.sign(&validator);

        assert_eq!(block.header.height, 1);
        assert!(block.verify_signature());
        assert!(block.verify_tx_root());
    }

    #[test]
    fn test_block_hash_deterministic() {
        let genesis = Block::genesis();
        assert_eq!(genesis.hash(), genesis.hash());
    }

    #[test]
    fn test_block_chain_links() {
        let validator = KeyPair::generate();
        let genesis = Block::genesis();

        let mut b1 = Block::new(
            &genesis,
            Vec::new(),
            [0u8; 32],
            validator.address(),
            validator.public_key(),
            100,
            1,
            5000000,
        );
        b1.sign(&validator);

        let mut b2 = Block::new(
            &b1,
            Vec::new(),
            [0u8; 32],
            validator.address(),
            validator.public_key(),
            200,
            1,
            5000000,
        );
        b2.sign(&validator);

        assert_eq!(b2.header.prev_hash, b1.hash());
        assert_eq!(b2.header.height, 2);
    }
}
