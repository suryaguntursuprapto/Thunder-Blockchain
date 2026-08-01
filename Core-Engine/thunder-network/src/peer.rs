// ---------------------------------------------------------------------------
//  Thunder Blockchain — Peer Management
// ---------------------------------------------------------------------------
//  Tracks connected peers and their state.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::time::Instant;

/// Unique peer identifier (derived from libp2p PeerId in production).
pub type PeerId = String;

/// Information about a connected peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: PeerId,
    pub address: String,
    pub connected_at: Instant,
    pub last_seen: Instant,
    pub block_height: u64,
}

/// Manages the set of connected peers.
pub struct PeerManager {
    peers: HashMap<PeerId, PeerInfo>,
    max_peers: usize,
}

impl PeerManager {
    pub fn new(max_peers: usize) -> Self {
        Self {
            peers: HashMap::new(),
            max_peers,
        }
    }

    /// Register a new peer connection.
    pub fn add_peer(&mut self, id: PeerId, address: String) -> bool {
        if self.peers.len() >= self.max_peers {
            return false;
        }
        let now = Instant::now();
        self.peers.insert(
            id.clone(),
            PeerInfo {
                id,
                address,
                connected_at: now,
                last_seen: now,
                block_height: 0,
            },
        );
        true
    }

    /// Remove a peer.
    pub fn remove_peer(&mut self, id: &str) {
        self.peers.remove(id);
    }

    /// Update the last-seen timestamp for a peer.
    pub fn touch(&mut self, id: &str) {
        if let Some(peer) = self.peers.get_mut(id) {
            peer.last_seen = Instant::now();
        }
    }

    /// Update a peer's known block height.
    pub fn update_height(&mut self, id: &str, height: u64) {
        if let Some(peer) = self.peers.get_mut(id) {
            peer.block_height = height;
        }
    }

    /// Get info about a specific peer.
    pub fn get(&self, id: &str) -> Option<&PeerInfo> {
        self.peers.get(id)
    }

    /// Number of connected peers.
    pub fn count(&self) -> usize {
        self.peers.len()
    }

    /// List all peer IDs.
    pub fn peer_ids(&self) -> Vec<&PeerId> {
        self.peers.keys().collect()
    }

    /// Returns true if connected to at least one peer.
    pub fn is_connected(&self) -> bool {
        !self.peers.is_empty()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_peer() {
        let mut pm = PeerManager::new(10);
        assert!(pm.add_peer("peer1".into(), "/ip4/127.0.0.1/tcp/4001".into()));
        assert_eq!(pm.count(), 1);
        pm.remove_peer("peer1");
        assert_eq!(pm.count(), 0);
    }

    #[test]
    fn test_max_peers() {
        let mut pm = PeerManager::new(2);
        assert!(pm.add_peer("p1".into(), "addr1".into()));
        assert!(pm.add_peer("p2".into(), "addr2".into()));
        assert!(!pm.add_peer("p3".into(), "addr3".into())); // rejected
    }
}
