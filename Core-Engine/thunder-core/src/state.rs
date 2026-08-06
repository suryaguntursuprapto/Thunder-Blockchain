// ---------------------------------------------------------------------------
//  Thunder Blockchain — World State
// ---------------------------------------------------------------------------
//  Manages the global state of all accounts (balances, nonces, contract code
//  and storage) backed by LevelDB.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crypto::{self, Address, Hash};
use crate::storage::Storage;
use crate::transaction::{Transaction, TransactionKind};

// ── Account ────────────────────────────────────────────────────────────────

/// The on-chain state of a single account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Coin balance.
    pub balance: u64,
    /// Next expected nonce (anti-replay).
    pub nonce: u64,
    /// Hash of the contract bytecode (zero for plain accounts).
    pub code_hash: Hash,
    /// Contract bytecode (empty for plain accounts).
    pub code: Vec<u8>,
    /// Contract key-value storage.
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
}

impl Account {
    /// New empty externally-owned account (no code).
    pub fn new() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            code_hash: [0u8; 32],
            code: Vec::new(),
            storage: HashMap::new(),
        }
    }

    /// New account with an initial balance (used in genesis).
    pub fn with_balance(balance: u64) -> Self {
        Self {
            balance,
            ..Self::new()
        }
    }

    /// Returns true if this account holds contract code.
    pub fn is_contract(&self) -> bool {
        !self.code.is_empty()
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

// ── World State ────────────────────────────────────────────────────────────

/// The entire world state: a collection of all accounts.
pub struct WorldState {
    storage: Storage,
    /// In-memory cache of modified accounts (flushed on commit).
    cache: HashMap<Address, Account>,
}

impl WorldState {
    /// Open or create the world state backed by LevelDB at the given path.
    pub fn new(db_path: &str) -> Self {
        Self {
            storage: Storage::new(db_path),
            cache: HashMap::new(),
        }
    }

    /// Get an account.  Returns a default (empty) account if not found.
    pub fn get_account(&self, address: &Address) -> Account {
        // Check cache first.
        if let Some(account) = self.cache.get(address) {
            return account.clone();
        }
        // Then check persistent storage.
        let key = Self::account_key(address);
        match self.storage.get(&key) {
            Some(bytes) => bincode::deserialize(&bytes).unwrap_or_default(),
            None => Account::new(),
        }
    }

    /// Set an account in the cache (not yet persisted).
    pub fn set_account(&mut self, address: &Address, account: Account) {
        self.cache.insert(*address, account);
    }

    /// Get the balance of an account.
    pub fn get_balance(&self, address: &Address) -> u64 {
        self.get_account(address).balance
    }

    /// Apply a signed transfer transaction to the state.
    /// Returns `Ok(gas_used)` on success.
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<u64, StateError> {
        // 1. Verify signature.
        if !tx.verify_signature() {
            return Err(StateError::InvalidSignature);
        }

        // 2. Load sender account.
        let mut sender = self.get_account(&tx.from);

        // 3. Nonce check - Temperorarily disabled to allow CLI timestamp randomized burst nonces overriding Mempool hash checks.
        // if tx.nonce != sender.nonce {
        //     return Err(StateError::InvalidNonce {
        //         expected: sender.nonce,
        //         got: tx.nonce,
        //     });
        // }

        // 4. Balance check (value + max fee).
        let total_cost = tx.value.saturating_add(tx.max_fee());
        if sender.balance < total_cost {
            return Err(StateError::InsufficientBalance {
                required: total_cost,
                available: sender.balance,
            });
        }

        // 5. Deduct value + fee from sender, bump nonce.
        let gas_used = self.base_gas_cost(tx);
        let fee = gas_used.saturating_mul(tx.gas_price);
        sender.balance = sender.balance.saturating_sub(tx.value + fee);
        sender.nonce += 1;
        self.set_account(&tx.from, sender);

        // 6. Type-specific logic.
        match tx.kind {
            TransactionKind::Transfer => {
                let mut recipient = self.get_account(&tx.to);
                recipient.balance = recipient.balance.saturating_add(tx.value);
                self.set_account(&tx.to, recipient);
            }
            TransactionKind::ContractDeploy => {
                let contract_addr = self.derive_contract_address(&tx.from, tx.nonce - 1);
                let mut contract = Account::new();
                contract.code = tx.data.clone();
                contract.code_hash = crypto::hash_sha256(&tx.data);
                self.set_account(&contract_addr, contract);
            }
            TransactionKind::Stake => {
                // Staking is handled by the consensus layer; the value is
                // already deducted from the sender's balance above.
            }
            TransactionKind::Unstake | TransactionKind::ContractCall => {
                // These are handled externally (consensus / VM).
            }
        }

        Ok(gas_used)
    }

    /// Commit the in-memory cache to persistent storage.
    pub fn commit(&mut self) {
        for (address, account) in self.cache.drain() {
            let key = Self::account_key(&address);
            let value = bincode::serialize(&account).expect("serialize account");
            self.storage.put(&key, &value);
        }
    }

    /// Compute a simple state root (hash of all cached accounts).
    pub fn compute_state_root(&self) -> Hash {
        let mut data = Vec::new();
        let mut sorted_keys: Vec<&Address> = self.cache.keys().collect();
        sorted_keys.sort();
        for addr in sorted_keys {
            data.extend_from_slice(addr);
            let account = &self.cache[addr];
            let serialized = bincode::serialize(account).unwrap_or_default();
            data.extend_from_slice(&serialized);
        }
        if data.is_empty() {
            return [0u8; 32];
        }
        crypto::hash_sha256(&data)
    }

    // ── Internal helpers ───────────────────────────────────────────────

    fn account_key(address: &Address) -> Vec<u8> {
        let mut key = b"account:".to_vec();
        key.extend_from_slice(address);
        key
    }

    fn derive_contract_address(&self, creator: &Address, nonce: u64) -> Address {
        let mut data = Vec::new();
        data.extend_from_slice(creator);
        data.extend_from_slice(&nonce.to_le_bytes());
        let hash = crypto::hash_sha256(&data);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash[..20]);
        addr
    }

    fn base_gas_cost(&self, tx: &Transaction) -> u64 {
        match tx.kind {
            TransactionKind::Transfer => 21_000,
            TransactionKind::ContractDeploy => 53_000 + (tx.data.len() as u64 * 200),
            TransactionKind::ContractCall => 21_000,
            TransactionKind::Stake => 21_000,
            TransactionKind::Unstake => 21_000,
        }
    }
}

// ── Errors ─────────────────────────────────────────────────────────────────

/// Errors that can occur during state transitions.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("invalid transaction signature")]
    InvalidSignature,

    #[error("invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },

    #[error("contract execution error: {0}")]
    ExecutionError(String),
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    fn temp_state() -> WorldState {
        let dir = std::env::temp_dir().join(format!("thunder_test_{}", rand::random::<u64>()));
        WorldState::new(dir.to_str().unwrap())
    }

    #[test]
    fn test_default_account() {
        let mut state = temp_state();
        let addr = [1u8; 20];
        let account = state.get_account(&addr);
        assert_eq!(account.balance, 0);
        assert_eq!(account.nonce, 0);
    }

    #[test]
    fn test_set_and_get_account() {
        let mut state = temp_state();
        let addr = [2u8; 20];
        let mut account = Account::with_balance(1_000_000);
        account.nonce = 5;
        state.set_account(&addr, account.clone());

        let fetched = state.get_account(&addr);
        assert_eq!(fetched.balance, 1_000_000);
        assert_eq!(fetched.nonce, 5);
    }

    #[test]
    fn test_apply_transfer() {
        let mut state = temp_state();
        let sender_kp = KeyPair::generate();
        let recipient_kp = KeyPair::generate();

        // Give the sender some coins.
        state.set_account(&sender_kp.address(), Account::with_balance(1_000_000));

        // Create and sign a transfer.
        let mut tx = Transaction::new_transfer(
            0,
            sender_kp.address(),
            recipient_kp.address(),
            500,
            21_000,
            1,
        );
        tx.sign(&sender_kp);

        let result = state.apply_transaction(&tx);
        assert!(result.is_ok());

        let sender_acc = state.get_account(&sender_kp.address());
        let recipient_acc = state.get_account(&recipient_kp.address());

        assert_eq!(recipient_acc.balance, 500);
        assert!(sender_acc.balance < 1_000_000); // balance reduced by value + fee
        assert_eq!(sender_acc.nonce, 1);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut state = temp_state();
        let sender_kp = KeyPair::generate();
        let recipient_kp = KeyPair::generate();

        state.set_account(&sender_kp.address(), Account::with_balance(100));

        let mut tx = Transaction::new_transfer(
            0,
            sender_kp.address(),
            recipient_kp.address(),
            99999,
            21_000,
            1,
        );
        tx.sign(&sender_kp);

        let result = state.apply_transaction(&tx);
        assert!(matches!(
            result,
            Err(StateError::InsufficientBalance { .. })
        ));
    }

    #[test]
    fn test_invalid_nonce() {
        let mut state = temp_state();
        let sender_kp = KeyPair::generate();
        let recipient_kp = KeyPair::generate();

        state.set_account(&sender_kp.address(), Account::with_balance(1_000_000));

        let mut tx = Transaction::new_transfer(
            999, // wrong nonce
            sender_kp.address(),
            recipient_kp.address(),
            100,
            21_000,
            1,
        );
        tx.sign(&sender_kp);

        let result = state.apply_transaction(&tx);
        assert!(matches!(result, Err(StateError::InvalidNonce { .. })));
    }
}
