// ---------------------------------------------------------------------------
//  Thunder Blockchain — P2P Network Protocol
// ---------------------------------------------------------------------------
//  Defines the message types exchanged between nodes and their serialisation.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

use thunder_core::block::Block;
use thunder_core::crypto::Hash;
use thunder_core::transaction::Transaction;
use thunder_consensus::types::Event;

/// Messages exchanged between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// A new transaction broadcast.
    NewTransaction(Transaction),

    /// A consensus event to be added to the DAG.
    SyncEvent(Event),

    /// Request events that this peer is missing.
    RequestEvents {
        /// Hashes of events the requester already has.
        known_hashes: Vec<Hash>,
    },

    /// Response with events the requester was missing.
    ResponseEvents {
        events: Vec<Event>,
    },

    /// A new finalised block announcement.
    NewBlock(Block),

    /// Request a block by height.
    RequestBlock {
        height: u64,
    },

    /// Response with the requested block.
    ResponseBlock {
        block: Option<Block>,
    },

    /// Ping (keepalive).
    Ping,

    /// Pong (keepalive response).
    Pong,
}

impl NetworkMessage {
    /// Serialise the message to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("serialize network message")
    }

    /// Deserialise from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}
