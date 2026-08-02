// ---------------------------------------------------------------------------
//  Thunder Blockchain — Asynchronous Byzantine Fault Tolerance (aBFT)
// ---------------------------------------------------------------------------
//  Implements the Hashgraph-inspired consensus algorithm:
//    1. divide_rounds  — assign round numbers to events
//    2. decide_fame    — determine which witnesses are "famous"
//    3. find_order     — establish total order of events with famous witnesses
//    4. extract_output — collect finalised transaction hashes
// ---------------------------------------------------------------------------

use std::collections::HashMap;

use thunder_core::crypto::{Address, Hash};

use crate::dag::Dag;
use crate::types::{ConsensusOutput, Event};

/// The main aBFT consensus engine.
pub struct AbftConsensus {
    /// The DAG of events from all validators.
    pub dag: Dag,
    /// Addresses of all validators participating in consensus.
    pub validators: Vec<Address>,
    /// The number of validators required for a supermajority (⌈2n/3⌉).
    pub supermajority: usize,
    /// The highest round that has been fully decided.
    pub last_decided_round: u64,
}

impl AbftConsensus {
    /// Create a new aBFT consensus engine for the given validator set.
    pub fn new(validators: Vec<Address>) -> Self {
        let n = validators.len();
        let supermajority = (n * 2).div_ceil(3); // ceiling of 2n/3
        Self {
            dag: Dag::new(),
            validators,
            supermajority,
            last_decided_round: 0,
        }
    }

    // ── Step 1: Divide Rounds ──────────────────────────────────────────

    /// Assign round numbers to all events in the DAG.
    ///
    /// An event's round is:
    ///   - 1        if it has no self-parent (first event by this creator)
    ///   - parent_round     if it cannot strongly-see a supermajority of
    ///     witnesses in `parent_round`
    ///   - parent_round + 1 otherwise
    ///
    /// Also marks each event as a *witness* if it is the first event by its
    /// creator in its round.
    pub fn divide_rounds(&mut self) {
        let event_hashes: Vec<Hash> = self.dag.all_events().iter().map(|e| e.hash).collect();
        let validators = self.validators.clone();

        for hash in &event_hashes {
            let Some(event) = self.dag.get(hash).cloned() else {
                continue;
            };
            let zero = [0u8; 32];

            let round = if event.self_parent == zero {
                // First event by this creator → round 1.
                1
            } else {
                let parent_round = self
                    .dag
                    .get(&event.self_parent)
                    .map(|e| e.round)
                    .unwrap_or(1);

                // Collect witnesses in parent_round.
                let witnesses: Vec<Hash> = self
                    .dag
                    .all_events()
                    .iter()
                    .filter(|e| e.round == parent_round && e.is_witness)
                    .map(|e| e.hash)
                    .collect();

                // Count how many witnesses this event can strongly see.
                let strongly_seen = witnesses
                    .iter()
                    .filter(|w| {
                        self.dag
                            .strongly_sees(hash, w, &validators, self.supermajority)
                    })
                    .count();

                if strongly_seen >= self.supermajority {
                    parent_round + 1
                } else {
                    parent_round
                }
            };

            // Check if this event is the first by its creator in this round.
            let is_witness = {
                let creator_events = self.dag.events_by_creator(&event.creator);
                !creator_events.iter().any(|h| {
                    if let Some(e) = self.dag.get(h) {
                        e.round == round && e.hash != event.hash && e.is_witness
                    } else {
                        false
                    }
                })
            };

            if let Some(ev) = self.dag.get_mut(hash) {
                ev.round = round;
                ev.is_witness = is_witness;
            }
        }
    }

    // ── Step 2: Decide Fame ────────────────────────────────────────────

    /// For each undecided witness, determine whether it is "famous".
    ///
    /// A witness in round `r` is famous if a supermajority of witnesses in
    /// round `r+1` can strongly-see it through distinct creators.
    pub fn decide_fame(&mut self) {
        let validators = self.validators.clone();
        let all_events: Vec<Event> = self.dag.all_events().iter().map(|e| (*e).clone()).collect();

        // Collect all undecided witnesses grouped by round.
        let mut witnesses_by_round: HashMap<u64, Vec<Hash>> = HashMap::new();
        for ev in &all_events {
            if ev.is_witness && ev.is_famous.is_none() {
                witnesses_by_round
                    .entry(ev.round)
                    .or_default()
                    .push(ev.hash);
            }
        }

        // Also collect already-known witnesses for strong-seeing.
        let mut all_witnesses_by_round: HashMap<u64, Vec<Hash>> = HashMap::new();
        for ev in &all_events {
            if ev.is_witness {
                all_witnesses_by_round
                    .entry(ev.round)
                    .or_default()
                    .push(ev.hash);
            }
        }

        let mut rounds: Vec<u64> = witnesses_by_round.keys().cloned().collect();
        rounds.sort();

        for round in &rounds {
            let next_round = round + 1;
            let next_witnesses = match all_witnesses_by_round.get(&next_round) {
                Some(ws) => ws.clone(),
                None => continue, // Can't decide yet — not enough events.
            };

            let undecided = match witnesses_by_round.get(round) {
                Some(ws) => ws.clone(),
                None => continue,
            };

            for witness_hash in &undecided {
                // Count how many next-round witnesses strongly-see this witness.
                let votes_for: usize = next_witnesses
                    .iter()
                    .filter(|nw| {
                        self.dag
                            .strongly_sees(nw, witness_hash, &validators, self.supermajority)
                    })
                    .count();

                let is_famous = votes_for >= self.supermajority;
                if let Some(ev) = self.dag.get_mut(witness_hash) {
                    ev.is_famous = Some(is_famous);
                }
            }
        }
    }

    // ── Step 3: Find Order ─────────────────────────────────────────────

    /// Establish a total order of events using the famous witnesses.
    ///
    /// An event receives a `round_received` equal to the first round in
    /// which all famous witnesses can see it.  Events are then sorted by
    /// (round_received, consensus_timestamp, hash) to produce a total order.
    pub fn find_order(&mut self) -> Vec<Hash> {
        let all_events: Vec<Event> = self.dag.all_events().iter().map(|e| (*e).clone()).collect();

        // Collect famous witnesses by round.
        let mut famous_by_round: HashMap<u64, Vec<Hash>> = HashMap::new();
        for ev in &all_events {
            if ev.is_witness && ev.is_famous == Some(true) {
                famous_by_round.entry(ev.round).or_default().push(ev.hash);
            }
        }

        let mut ordered: Vec<(u64, u64, Hash)> = Vec::new();

        // For each unordered event, find the earliest round where all famous
        // witnesses can see it.
        for ev in &all_events {
            if ev.round_received.is_some() {
                continue; // Already ordered.
            }

            let mut rounds: Vec<u64> = famous_by_round.keys().cloned().collect();
            rounds.sort();

            for round in &rounds {
                if *round <= ev.round {
                    continue; // Famous witnesses must be in a later round.
                }

                let famous = &famous_by_round[round];
                let all_see = famous.iter().all(|fw| self.dag.is_ancestor(&ev.hash, fw));

                if all_see {
                    // Consensus timestamp = median of famous witnesses' timestamps.
                    let mut timestamps: Vec<u64> = famous
                        .iter()
                        .filter_map(|fw| self.dag.get(fw))
                        .map(|e| e.timestamp)
                        .collect();
                    timestamps.sort();
                    let median_ts = timestamps[timestamps.len() / 2];

                    if let Some(e) = self.dag.get_mut(&ev.hash) {
                        e.round_received = Some(*round);
                        e.consensus_timestamp = median_ts;
                    }

                    ordered.push((*round, median_ts, ev.hash));
                    break;
                }
            }
        }

        // Sort by (round_received, consensus_timestamp, hash) for total order.
        ordered.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

        ordered.iter().map(|(_, _, h)| *h).collect()
    }

    // ── Step 4: Extract Output ─────────────────────────────────────────

    /// Run all consensus steps and extract newly-ordered transaction hashes.
    pub fn process(&mut self) -> Option<ConsensusOutput> {
        self.divide_rounds();
        self.decide_fame();
        let ordered = self.find_order();

        if ordered.is_empty() {
            return None;
        }

        let mut tx_hashes = Vec::new();
        let mut max_round = self.last_decided_round;

        for hash in &ordered {
            if let Some(ev) = self.dag.get(hash) {
                if let Some(rr) = ev.round_received {
                    if rr > self.last_decided_round {
                        tx_hashes.extend_from_slice(&ev.payload);
                        max_round = max_round.max(rr);
                    }
                }
            }
        }

        if tx_hashes.is_empty() {
            return None;
        }

        self.last_decided_round = max_round;

        Some(ConsensusOutput {
            ordered_tx_hashes: tx_hashes,
            round_decided: max_round,
        })
    }

    /// Add an event to the consensus DAG.
    pub fn add_event(&mut self, event: Event) -> Result<(), crate::dag::DagError> {
        self.dag.insert(event)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Event;
    use thunder_core::crypto::{hash_sha256, KeyPair};

    fn make_signed_event(
        kp: &KeyPair,
        self_parent: Hash,
        other_parent: Hash,
        payload: Vec<Hash>,
        ts: u64,
    ) -> Event {
        let mut ev = Event::new(kp.address(), self_parent, other_parent, payload, ts);
        ev.hash = hash_sha256(&ev.signable_bytes());
        ev.signature = kp.sign(&ev.hash);
        ev
    }

    #[test]
    fn test_consensus_creation() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let kp3 = KeyPair::generate();
        let validators = vec![kp1.address(), kp2.address(), kp3.address()];
        let consensus = AbftConsensus::new(validators.clone());
        assert_eq!(consensus.validators.len(), 3);
        assert_eq!(consensus.supermajority, 2);
    }

    #[test]
    fn test_divide_rounds_basic() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let validators = vec![kp1.address(), kp2.address()];
        let mut consensus = AbftConsensus::new(validators);

        let ev1 = make_signed_event(&kp1, [0u8; 32], [0u8; 32], vec![], 1);
        let ev2 = make_signed_event(&kp2, [0u8; 32], [0u8; 32], vec![], 2);

        consensus.add_event(ev1).unwrap();
        consensus.add_event(ev2).unwrap();

        consensus.divide_rounds();

        // First events should be round 1 and be witnesses.
        for ev in consensus.dag.all_events() {
            assert_eq!(ev.round, 1);
            assert!(ev.is_witness);
        }
    }

    #[test]
    fn test_add_event() {
        let kp = KeyPair::generate();
        let validators = vec![kp.address()];
        let mut consensus = AbftConsensus::new(validators);

        let ev = make_signed_event(&kp, [0u8; 32], [0u8; 32], vec![[42u8; 32]], 1);
        consensus.add_event(ev).unwrap();

        assert_eq!(consensus.dag.len(), 1);
    }
}
