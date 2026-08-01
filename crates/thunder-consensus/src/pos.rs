// ---------------------------------------------------------------------------
//  Thunder Blockchain — Proof of Stake
// ---------------------------------------------------------------------------
//  Manages the validator set: registration, stake-weighted selection,
//  slashing, and reward distribution.
// ---------------------------------------------------------------------------

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use thunder_core::crypto::{Address, Hash, PublicKey};

use crate::types::Validator;

// ── Validator Set ──────────────────────────────────────────────────────────

/// Manages the active set of validators and the staking ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    /// All registered validators, keyed by address.
    validators: HashMap<Address, Validator>,
    /// Minimum stake required to become a validator.
    pub min_stake: u64,
    /// Total staked coins across all validators.
    total_stake: u64,
}

impl ValidatorSet {
    /// Create a new, empty validator set with the given minimum stake.
    pub fn new(min_stake: u64) -> Self {
        Self {
            validators: HashMap::new(),
            min_stake,
            total_stake: 0,
        }
    }

    // ── Registration ───────────────────────────────────────────────────

    /// Register a new validator.  Fails if the stake is below the minimum.
    pub fn register(
        &mut self,
        address: Address,
        public_key: PublicKey,
        stake: u64,
    ) -> Result<(), StakeError> {
        if stake < self.min_stake {
            return Err(StakeError::BelowMinimumStake {
                minimum: self.min_stake,
                provided: stake,
            });
        }
        if self.validators.contains_key(&address) {
            return Err(StakeError::AlreadyRegistered);
        }

        self.total_stake += stake;
        self.validators
            .insert(address, Validator::new(address, public_key, stake));
        Ok(())
    }

    /// Increase the stake of an existing validator.
    pub fn add_stake(&mut self, address: &Address, amount: u64) -> Result<(), StakeError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(StakeError::NotRegistered)?;
        v.stake += amount;
        self.total_stake += amount;
        Ok(())
    }

    /// Remove a validator from the active set and return their stake.
    pub fn unregister(&mut self, address: &Address) -> Result<u64, StakeError> {
        let v = self
            .validators
            .remove(address)
            .ok_or(StakeError::NotRegistered)?;
        self.total_stake -= v.stake;
        Ok(v.stake)
    }

    // ── Selection ──────────────────────────────────────────────────────

    /// Select a block proposer using stake-weighted deterministic randomness.
    ///
    /// The `seed` should be derived from the previous block hash or a VRF
    /// output to ensure fairness.
    pub fn select_proposer(&self, seed: &Hash) -> Option<Address> {
        let active: Vec<&Validator> = self.active_validators();
        if active.is_empty() {
            return None;
        }

        // Sum all active stakes.
        let total: u64 = active.iter().map(|v| v.stake).sum();
        if total == 0 {
            return None;
        }

        // Use the seed to derive a pseudo-random index.
        let mut rng = Self::seeded_rng(seed);
        let target: u64 = rng.gen_range(0..total);

        let mut cumulative = 0u64;
        for v in &active {
            cumulative += v.stake;
            if target < cumulative {
                return Some(v.address);
            }
        }

        // Fallback (should not happen).
        Some(active.last().unwrap().address)
    }

    /// Get all active validators sorted by stake (descending).
    pub fn active_validators(&self) -> Vec<&Validator> {
        let mut vs: Vec<&Validator> = self.validators.values().filter(|v| v.is_active).collect();
        vs.sort_by(|a, b| b.stake.cmp(&a.stake));
        vs
    }

    /// Total number of registered validators.
    pub fn count(&self) -> usize {
        self.validators.len()
    }

    /// Total number of active validators.
    pub fn active_count(&self) -> usize {
        self.validators.values().filter(|v| v.is_active).count()
    }

    /// Get a validator by address.
    pub fn get(&self, address: &Address) -> Option<&Validator> {
        self.validators.get(address)
    }

    /// Total amount of coins staked.
    pub fn total_stake(&self) -> u64 {
        self.total_stake
    }

    /// The supermajority threshold (⅔ of total stake).
    pub fn supermajority(&self) -> u64 {
        (self.total_stake * 2) / 3 + 1
    }

    // ── Slashing ───────────────────────────────────────────────────────

    /// Slash a validator's stake by `amount` (e.g. for double-signing).
    pub fn slash(&mut self, address: &Address, amount: u64) -> Result<u64, StakeError> {
        let v = self
            .validators
            .get_mut(address)
            .ok_or(StakeError::NotRegistered)?;
        let slash_amount = amount.min(v.stake);
        v.stake -= slash_amount;
        self.total_stake -= slash_amount;

        // Deactivate if stake falls below minimum.
        if v.stake < self.min_stake {
            v.is_active = false;
        }

        Ok(slash_amount)
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Build a simple seeded RNG from a hash (ChaCha-like via `rand`).
    fn seeded_rng(seed: &Hash) -> impl Rng {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        StdRng::from_seed(*seed)
    }
}

// ── Errors ─────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum StakeError {
    #[error("stake {provided} is below minimum {minimum}")]
    BelowMinimumStake { minimum: u64, provided: u64 },

    #[error("validator already registered")]
    AlreadyRegistered,

    #[error("validator not registered")]
    NotRegistered,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use thunder_core::crypto::KeyPair;

    fn make_validator_set() -> (ValidatorSet, Vec<KeyPair>) {
        let mut vs = ValidatorSet::new(100);
        let mut kps = Vec::new();

        for stake in [1000u64, 2000, 3000] {
            let kp = KeyPair::generate();
            vs.register(kp.address(), kp.public_key(), stake)
                .unwrap();
            kps.push(kp);
        }

        (vs, kps)
    }

    #[test]
    fn test_register_and_count() {
        let (vs, _) = make_validator_set();
        assert_eq!(vs.count(), 3);
        assert_eq!(vs.active_count(), 3);
        assert_eq!(vs.total_stake(), 6000);
    }

    #[test]
    fn test_below_minimum_stake() {
        let mut vs = ValidatorSet::new(100);
        let kp = KeyPair::generate();
        let result = vs.register(kp.address(), kp.public_key(), 50);
        assert!(matches!(result, Err(StakeError::BelowMinimumStake { .. })));
    }

    #[test]
    fn test_select_proposer_deterministic() {
        let (vs, _) = make_validator_set();
        let seed = [42u8; 32];
        let p1 = vs.select_proposer(&seed);
        let p2 = vs.select_proposer(&seed);
        assert_eq!(p1, p2); // Same seed → same proposer.
    }

    #[test]
    fn test_slashing() {
        let (mut vs, kps) = make_validator_set();
        let target = kps[0].address();
        let original_stake = vs.get(&target).unwrap().stake;
        let slashed = vs.slash(&target, 500).unwrap();
        assert_eq!(slashed, 500);
        assert_eq!(vs.get(&target).unwrap().stake, original_stake - 500);
    }

    #[test]
    fn test_slash_deactivates_below_minimum() {
        let (mut vs, kps) = make_validator_set();
        let target = kps[0].address();
        vs.slash(&target, 950).unwrap(); // 1000 - 950 = 50 < min_stake(100)
        assert!(!vs.get(&target).unwrap().is_active);
    }

    #[test]
    fn test_unregister() {
        let (mut vs, kps) = make_validator_set();
        let returned = vs.unregister(&kps[1].address()).unwrap();
        assert_eq!(returned, 2000);
        assert_eq!(vs.count(), 2);
    }

    #[test]
    fn test_supermajority() {
        let (vs, _) = make_validator_set();
        assert_eq!(vs.supermajority(), 4001); // (6000 * 2 / 3) + 1
    }
}
