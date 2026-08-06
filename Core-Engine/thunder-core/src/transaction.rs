// ---------------------------------------------------------------------------
//  Thunder Blockchain — Transaction
// ---------------------------------------------------------------------------
//  Defines the `Transaction` type and its variants (Transfer, ContractDeploy,
//  ContractCall).  Every transaction is signed by the sender and carries
//  a nonce to prevent replay attacks.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::crypto::{self, Address, Hash, PublicKey, Signature};

// ── Transaction Kind ───────────────────────────────────────────────────────

/// The purpose of a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionKind {
    /// Plain coin transfer.
    Transfer,
    /// Deploy a new smart contract (data = bytecode).
    ContractDeploy,
    /// Call an existing smart contract function.
    ContractCall,
    /// Stake coins to become a validator.
    Stake,
    /// Unstake coins (withdraw from validator set).
    Unstake,
}

// ── Transaction ────────────────────────────────────────────────────────────

/// A single transaction on the Thunder Blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Chain ID for replay protection (e.g., 1 = Mainnet, 2 = Testnet)
    pub chain_id: u64,
    /// Sender nonce (monotonically increasing per account).
    pub nonce: u64,
    /// Sender address.
    pub from: Address,
    /// Recipient address (zero-address for contract deployment).
    pub to: Address,
    /// Amount of coins to transfer.
    pub value: u64,
    /// Arbitrary payload (bytecode for deploys, call-data for calls).
    pub data: Vec<u8>,
    /// Maximum gas units the sender is willing to pay.
    pub gas_limit: u64,
    /// Price per gas unit (in smallest coin denomination).
    pub gas_price: u64,
    /// Transaction variant.
    pub kind: TransactionKind,
    /// Ed25519 signature over the transaction hash.
    #[serde(with = "BigArray")]
    pub signature: Signature,
    /// Sender's public key (needed for signature verification).
    pub public_key: PublicKey,
}

impl Transaction {
    // ── Construction helpers ────────────────────────────────────────────

    /// Create a new **unsigned** transfer transaction.
    pub fn new_transfer(
        chain_id: u64,
        nonce: u64,
        from: Address,
        to: Address,
        value: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            chain_id,
            nonce,
            from,
            to,
            value,
            data: Vec::new(),
            gas_limit,
            gas_price,
            kind: TransactionKind::Transfer,
            signature: [0u8; 64],
            public_key: [0u8; 32],
        }
    }

    /// Create a new **unsigned** contract-deploy transaction.
    pub fn new_deploy(
        chain_id: u64,
        nonce: u64,
        from: Address,
        bytecode: Vec<u8>,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            chain_id,
            nonce,
            from,
            to: [0u8; 20], // zero-address → deploy
            value: 0,
            data: bytecode,
            gas_limit,
            gas_price,
            kind: TransactionKind::ContractDeploy,
            signature: [0u8; 64],
            public_key: [0u8; 32],
        }
    }

    /// Create a new **unsigned** contract-call transaction.
    pub fn new_call(
        chain_id: u64,
        nonce: u64,
        from: Address,
        to: Address,
        value: u64,
        call_data: Vec<u8>,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            chain_id,
            nonce,
            from,
            to,
            value,
            data: call_data,
            gas_limit,
            gas_price,
            kind: TransactionKind::ContractCall,
            signature: [0u8; 64],
            public_key: [0u8; 32],
        }
    }

    /// Create a new **unsigned** stake transaction.
    pub fn new_stake(
        chain_id: u64,
        nonce: u64,
        from: Address,
        amount: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            chain_id,
            nonce,
            from,
            to: [0u8; 20],
            value: amount,
            data: Vec::new(),
            gas_limit,
            gas_price,
            kind: TransactionKind::Stake,
            signature: [0u8; 64],
            public_key: [0u8; 32],
        }
    }

    // ── Hashing & Signing ──────────────────────────────────────────────

    /// Compute the bytes that are signed (everything except signature).
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.chain_id.to_le_bytes());
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf.extend_from_slice(&self.from);
        buf.extend_from_slice(&self.to);
        buf.extend_from_slice(&self.value.to_le_bytes());
        buf.extend_from_slice(&self.data);
        buf.extend_from_slice(&self.gas_limit.to_le_bytes());
        buf.extend_from_slice(&self.gas_price.to_le_bytes());
        buf.extend_from_slice(&self.kind_id().to_le_bytes());
        buf
    }

    /// Compute the transaction hash (SHA-256 of the signable bytes).
    pub fn hash(&self) -> Hash {
        crypto::hash_sha256(&self.signable_bytes())
    }

    /// Sign the transaction in-place with the given key pair.
    pub fn sign(&mut self, key_pair: &crypto::KeyPair) {
        self.public_key = key_pair.public_key();
        self.from = key_pair.address();
        let hash = self.hash();
        self.signature = key_pair.sign(&hash);
    }

    /// Verify the attached signature.
    pub fn verify_signature(&self) -> bool {
        // Check that the from-address matches the public key.
        if self.from != crypto::address_from_public_key(&self.public_key) {
            return false;
        }
        let hash = self.hash();
        crypto::verify_signature(&self.public_key, &hash, &self.signature)
    }

    // ── Helpers ────────────────────────────────────────────────────────

    /// Numeric id for the transaction kind (used in serialisation).
    fn kind_id(&self) -> u8 {
        match self.kind {
            TransactionKind::Transfer => 0,
            TransactionKind::ContractDeploy => 1,
            TransactionKind::ContractCall => 2,
            TransactionKind::Stake => 3,
            TransactionKind::Unstake => 4,
        }
    }

    /// Total fee the sender is willing to pay = gas_limit × gas_price.
    pub fn max_fee(&self) -> u64 {
        self.gas_limit.saturating_mul(self.gas_price)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_transfer_sign_verify() {
        let sender = KeyPair::generate();
        let recipient = KeyPair::generate();
        let mut tx =
            Transaction::new_transfer(1, 0, sender.address(), recipient.address(), 1000, 21000, 1);
        tx.sign(&sender);
        assert!(tx.verify_signature());
    }

    #[test]
    fn test_tampered_tx_fails_verification() {
        let sender = KeyPair::generate();
        let recipient = KeyPair::generate();
        let mut tx =
            Transaction::new_transfer(1, 0, sender.address(), recipient.address(), 1000, 21000, 1);
        tx.sign(&sender);

        // Tamper with the value after signing
        tx.value = 9999;
        assert!(!tx.verify_signature());
    }

    #[test]
    fn test_deploy_tx() {
        let sender = KeyPair::generate();
        let bytecode = vec![0x01, 0x02, 0x03, 0x04];
        let mut tx = Transaction::new_deploy(1, 0, sender.address(), bytecode.clone(), 100_000, 1);
        tx.sign(&sender);
        assert!(tx.verify_signature());
        assert_eq!(tx.kind, TransactionKind::ContractDeploy);
        assert_eq!(tx.data, bytecode);
    }

    #[test]
    fn test_hash_deterministic() {
        let sender = KeyPair::generate();
        let recipient = KeyPair::generate();
        let tx = Transaction::new_transfer(1, 0, sender.address(), recipient.address(), 100, 21000, 1);
        assert_eq!(tx.hash(), tx.hash());
    }
}
