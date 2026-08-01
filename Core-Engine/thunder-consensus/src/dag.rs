// ---------------------------------------------------------------------------
//  Thunder Blockchain — DAG (Directed Acyclic Graph)
// ---------------------------------------------------------------------------
//  The DAG stores events created by validators.  Each event references two
//  parents: a self-parent (previous event by the same creator) and an
//  other-parent (an event from a different creator).
// ---------------------------------------------------------------------------

use std::collections::{HashMap, HashSet};

use thunder_core::crypto::{Address, Hash};

use crate::types::Event;

/// The Hashgraph-style DAG.
pub struct Dag {
    /// All events, keyed by their hash.
    events: HashMap<Hash, Event>,
    /// For each creator, an ordered list of their event hashes.
    creator_events: HashMap<Address, Vec<Hash>>,
    /// Set of event hashes that are "tips" (no child references them yet).
    tips: HashSet<Hash>,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            creator_events: HashMap::new(),
            tips: HashSet::new(),
        }
    }

    // ── Insertion ──────────────────────────────────────────────────────

    /// Insert a new event into the DAG.
    ///
    /// The caller must have already computed the event's hash and signature.
    /// Returns `Err` on duplicate hash or missing parents.
    pub fn insert(&mut self, event: Event) -> Result<(), DagError> {
        if self.events.contains_key(&event.hash) {
            return Err(DagError::DuplicateEvent);
        }

        // Verify parents exist (zero-hash means "no parent" for the first event).
        let zero = [0u8; 32];
        if event.self_parent != zero && !self.events.contains_key(&event.self_parent) {
            return Err(DagError::MissingSelfParent);
        }
        if event.other_parent != zero && !self.events.contains_key(&event.other_parent) {
            return Err(DagError::MissingOtherParent);
        }

        // Update tips: remove parents from tips, add this event.
        if event.self_parent != zero {
            self.tips.remove(&event.self_parent);
        }
        if event.other_parent != zero {
            self.tips.remove(&event.other_parent);
        }
        self.tips.insert(event.hash);

        // Track per-creator ordering.
        self.creator_events
            .entry(event.creator)
            .or_default()
            .push(event.hash);

        self.events.insert(event.hash, event);
        Ok(())
    }

    // ── Queries ────────────────────────────────────────────────────────

    /// Get an event by hash.
    pub fn get(&self, hash: &Hash) -> Option<&Event> {
        self.events.get(hash)
    }

    /// Get a mutable reference to an event by hash.
    pub fn get_mut(&mut self, hash: &Hash) -> Option<&mut Event> {
        self.events.get_mut(hash)
    }

    /// The latest event created by this address.
    pub fn latest_event(&self, creator: &Address) -> Option<&Event> {
        self.creator_events
            .get(creator)
            .and_then(|evs| evs.last())
            .and_then(|h| self.events.get(h))
    }

    /// All event hashes created by this address (in insertion order).
    pub fn events_by_creator(&self, creator: &Address) -> &[Hash] {
        self.creator_events
            .get(creator)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Return current tip hashes (events not yet referenced as parents).
    pub fn tips(&self) -> &HashSet<Hash> {
        &self.tips
    }

    /// Total number of events in the DAG.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Check whether event `x` is an ancestor of event `y`.
    ///
    /// Uses BFS backwards through parent links.
    pub fn is_ancestor(&self, x: &Hash, y: &Hash) -> bool {
        if x == y {
            return true;
        }
        let zero = [0u8; 32];
        let mut visited = HashSet::new();
        let mut queue = vec![*y];

        while let Some(current) = queue.pop() {
            if current == *x {
                return true;
            }
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(ev) = self.events.get(&current) {
                if ev.self_parent != zero {
                    queue.push(ev.self_parent);
                }
                if ev.other_parent != zero {
                    queue.push(ev.other_parent);
                }
            }
        }

        false
    }

    /// Check if event `x` can *strongly see* event `y` via a supermajority
    /// of distinct creators.
    ///
    /// `supermajority_count` is typically ⌈2n/3⌉ where n = number of
    /// validators.
    pub fn strongly_sees(
        &self,
        x: &Hash,
        y: &Hash,
        validators: &[Address],
        supermajority_count: usize,
    ) -> bool {
        // For each validator, check whether there exists an event by that
        // validator such that x → e → y (i.e., x sees e, and e sees y).
        let mut count = 0;
        for creator in validators {
            for ev_hash in self.events_by_creator(creator) {
                // x can reach ev, and ev can reach y.
                if self.is_ancestor(ev_hash, x) && self.is_ancestor(y, ev_hash) {
                    count += 1;
                    break;
                }
            }
            if count >= supermajority_count {
                return true;
            }
        }
        false
    }

    /// Return all events, ordered by insertion (useful for iteration).
    pub fn all_events(&self) -> Vec<&Event> {
        let mut all: Vec<&Event> = self.events.values().collect();
        all.sort_by_key(|e| e.timestamp);
        all
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

// ── Errors ─────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum DagError {
    #[error("duplicate event hash")]
    DuplicateEvent,
    #[error("self-parent not found in DAG")]
    MissingSelfParent,
    #[error("other-parent not found in DAG")]
    MissingOtherParent,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use thunder_core::crypto::{hash_sha256, KeyPair};

    fn make_event(creator: Address, self_parent: Hash, other_parent: Hash, ts: u64) -> Event {
        let mut ev = Event::new(creator, self_parent, other_parent, vec![], ts);
        ev.hash = hash_sha256(&ev.signable_bytes());
        ev
    }

    #[test]
    fn test_insert_and_get() {
        let mut dag = Dag::new();
        let kp = KeyPair::generate();
        let ev = make_event(kp.address(), [0u8; 32], [0u8; 32], 1);
        let hash = ev.hash;
        dag.insert(ev).unwrap();
        assert!(dag.get(&hash).is_some());
        assert_eq!(dag.len(), 1);
    }

    #[test]
    fn test_duplicate_insert_fails() {
        let mut dag = Dag::new();
        let kp = KeyPair::generate();
        let ev = make_event(kp.address(), [0u8; 32], [0u8; 32], 1);
        dag.insert(ev.clone()).unwrap();
        assert!(matches!(dag.insert(ev), Err(DagError::DuplicateEvent)));
    }

    #[test]
    fn test_ancestry() {
        let mut dag = Dag::new();
        let kp = KeyPair::generate();

        let ev1 = make_event(kp.address(), [0u8; 32], [0u8; 32], 1);
        let h1 = ev1.hash;
        dag.insert(ev1).unwrap();

        let ev2 = make_event(kp.address(), h1, [0u8; 32], 2);
        let h2 = ev2.hash;
        dag.insert(ev2).unwrap();

        assert!(dag.is_ancestor(&h1, &h2)); // h1 is ancestor of h2
        assert!(!dag.is_ancestor(&h2, &h1)); // h2 is NOT ancestor of h1
    }

    #[test]
    fn test_tips() {
        let mut dag = Dag::new();
        let kp = KeyPair::generate();

        let ev1 = make_event(kp.address(), [0u8; 32], [0u8; 32], 1);
        let h1 = ev1.hash;
        dag.insert(ev1).unwrap();
        assert!(dag.tips().contains(&h1));

        let ev2 = make_event(kp.address(), h1, [0u8; 32], 2);
        let h2 = ev2.hash;
        dag.insert(ev2).unwrap();
        assert!(!dag.tips().contains(&h1)); // h1 no longer a tip
        assert!(dag.tips().contains(&h2));
    }

    #[test]
    fn test_creator_events() {
        let mut dag = Dag::new();
        let kp = KeyPair::generate();

        let ev1 = make_event(kp.address(), [0u8; 32], [0u8; 32], 1);
        let h1 = ev1.hash;
        dag.insert(ev1).unwrap();

        let ev2 = make_event(kp.address(), h1, [0u8; 32], 2);
        dag.insert(ev2).unwrap();

        assert_eq!(dag.events_by_creator(&kp.address()).len(), 2);
    }
}
