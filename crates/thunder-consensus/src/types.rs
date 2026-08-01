// ---------------------------------------------------------------------------
//  Thunder Blockchain — Consensus Types
// ---------------------------------------------------------------------------
//  Shared type definitions for the PoS aBFT consensus engine.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use thunder_core::crypto::{Address, Hash, PublicKey, Signature};

// ── Validator ──────────────────────────────────────────────────────────────

/// A registered validator in the Proof-of-Stake system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// On-chain address.
    pub address: Address,
    /// Ed25519 public key.
    pub public_key: PublicKey,
    /// Amount of coins staked.
    pub stake: u64,
    /// Whether this validator is currently active.
    pub is_active: bool,
}

impl Validator {
    pub fn new(address: Address, public_key: PublicKey, stake: u64) -> Self {
        Self {
            address,
            public_key,
            stake,
            is_active: true,
        }
    }
}

// ── DAG Event ──────────────────────────────────────────────────────────────

/// A single event in the aBFT Directed Acyclic Graph.
///
/// Each event references two parents (self-parent from the same creator and
/// an other-parent from a different creator) and optionally carries
/// transaction payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique hash identifying this event.
    pub hash: Hash,
    /// Address of the validator who created this event.
    pub creator: Address,
    /// Hash of the creator's previous event (self-parent).  Zero for the
    /// first event by this creator.
    pub self_parent: Hash,
    /// Hash of an event from another validator (other-parent).  Zero for
    /// the very first event in the DAG.
    pub other_parent: Hash,
    /// Transaction hashes bundled into this event.
    pub payload: Vec<Hash>,
    /// Consensus round number (computed by `divide_rounds`).
    pub round: u64,
    /// Whether this event is a *witness* (first event by its creator in its
    /// round).
    pub is_witness: bool,
    /// Fame status: `None` = undecided, `Some(true)` = famous,
    /// `Some(false)` = not famous.
    pub is_famous: Option<bool>,
    /// Consensus timestamp once ordering is finalised.
    pub consensus_timestamp: u64,
    /// The real-world timestamp when the event was created.
    pub timestamp: u64,
    /// Ed25519 signature over the event hash.
    #[serde(with = "BigArray")]
    pub signature: Signature,
    /// Topological round-received (set during `find_order`).
    pub round_received: Option<u64>,
}

impl Event {
    /// Create a new event (hash and signature are computed externally).
    pub fn new(
        creator: Address,
        self_parent: Hash,
        other_parent: Hash,
        payload: Vec<Hash>,
        timestamp: u64,
    ) -> Self {
        Self {
            hash: [0u8; 32], // filled in later
            creator,
            self_parent,
            other_parent,
            payload,
            round: 0,
            is_witness: false,
            is_famous: None,
            consensus_timestamp: 0,
            timestamp,
            signature: [0u8; 64],
            round_received: None,
        }
    }

    /// Compute the bytes to be hashed / signed.
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.creator);
        buf.extend_from_slice(&self.self_parent);
        buf.extend_from_slice(&self.other_parent);
        for h in &self.payload {
            buf.extend_from_slice(h);
        }
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf
    }
}

// ── Consensus Result ───────────────────────────────────────────────────────

/// The output of the consensus engine after processing events.
#[derive(Debug, Clone)]
pub struct ConsensusOutput {
    /// Transaction hashes in the finalised total order.
    pub ordered_tx_hashes: Vec<Hash>,
    /// The consensus round that was just decided.
    pub round_decided: u64,
}
