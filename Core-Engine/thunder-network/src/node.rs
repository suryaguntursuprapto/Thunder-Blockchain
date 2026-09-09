// ---------------------------------------------------------------------------
//  Thunder Blockchain — Node
// ---------------------------------------------------------------------------
//  The main node struct that ties together networking, consensus, state, and
//  the VM.  In production this runs an async event loop; here we provide the
//  synchronous building blocks.
// ---------------------------------------------------------------------------

use std::sync::{Arc, RwLock};
use thunder_consensus::abft::AbftConsensus;
use thunder_consensus::pos::ValidatorSet;
use thunder_consensus::types::Event;
use thunder_core::block::Block;
use thunder_core::crypto::{self, KeyPair};
use thunder_core::state::WorldState;
use thunder_core::transaction::Transaction;

use crate::peer::PeerManager;

/// Configuration for a node.
pub struct NodeConfig {
    pub data_dir: String,
    pub listen_port: u16,
    pub max_peers: usize,
    pub min_stake: u64,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            listen_port: 30303,
            max_peers: 50,
            min_stake: 1000,
        }
    }
}

/// A Thunder Blockchain node.
pub struct Node {
    /// This node's key pair (validator identity).
    pub key_pair: KeyPair,
    /// World state (accounts, contracts).
    pub state: Arc<RwLock<WorldState>>,
    /// The blockchain (ordered list of finalised blocks).
    pub chain: Vec<Block>,
    /// Transaction mempool (pending transactions).
    pub mempool: Vec<Transaction>,
    /// Consensus engine.
    pub consensus: AbftConsensus,
    /// Validator set.
    pub validator_set: ValidatorSet,
    /// Peer manager.
    pub peers: PeerManager,
    /// Node configuration.
    pub config: NodeConfig,
}

impl Node {
    /// Create a new node with the given key pair and configuration.
    pub fn new(key_pair: KeyPair, config: NodeConfig) -> Self {
        let state = Arc::new(RwLock::new(WorldState::new(&format!(
            "{}/state",
            config.data_dir
        ))));
        
        let genesis = Block::genesis();
        let validator_set = ValidatorSet::new(config.min_stake);
        let consensus = AbftConsensus::new(Vec::new());
        let peers = PeerManager::new(config.max_peers);

        Self {
            key_pair,
            state,
            chain: vec![genesis],
            mempool: Vec::new(),
            consensus,
            validator_set,
            peers,
            config,
        }
    }

    /// Get the latest block in the chain.
    pub fn latest_block(&self) -> &Block {
        self.chain.last().expect("chain should never be empty")
    }

    /// Current chain height.
    pub fn height(&self) -> u64 {
        self.latest_block().header.height
    }

    /// Add a transaction to the mempool.
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        if !tx.verify_signature() {
            return Err("invalid transaction signature".to_string());
        }

        // Strict Balance Pre-Flight Check preventing empty wallets from bypassing EVM
        let sender = self.state.read().unwrap().get_account(&tx.from);
        let total_cost = tx.value.saturating_add(tx.max_fee());
        if sender.balance < total_cost {
            return Err(format!(
                "Insufficient Balance. Required: {:.9} THDR (Amount: {:.9} + Gas: {:.9}), Available: {:.9} THDR",
                total_cost as f64 / 1_000_000_000.0,
                tx.value as f64 / 1_000_000_000.0,
                tx.max_fee() as f64 / 1_000_000_000.0,
                sender.balance as f64 / 1_000_000_000.0
            ));
        }

        self.mempool.push(tx);
        Ok(())
    }

    /// Register this node as a validator.
    pub fn register_as_validator(&mut self, stake: u64) -> Result<(), String> {
        self.validator_set
            .register(self.key_pair.address(), self.key_pair.public_key(), stake, 0)
            .map_err(|e| e.to_string())?;

        // Update consensus engine with the new validator set.
        self.consensus.validators = self
            .validator_set
            .active_validators()
            .iter()
            .map(|v| v.address)
            .collect();

        // If we are at genesis block, inject genesis transactions for Staking Pool
        if self.chain.len() == 1 {
            let mut state = self.state.write().unwrap();
            
            let validator_addr = self.key_pair.address();

            // 1. Mint balance to the genesis validator so they can deploy and stake
            let mut genesis_validator = state.get_account(&validator_addr);
            genesis_validator.balance = 1_000_000_000_000_000;
            state.set_account(&validator_addr, genesis_validator);

            let possible_paths = [
                "Core-Engine/contracts/staking-pool/StakingPool.ths",
                "contracts/staking-pool/StakingPool.ths",
                "../Core-Engine/contracts/staking-pool/StakingPool.ths",
            ];
            
            let mut contract_source = None;
            for path in &possible_paths {
                if let Ok(source) = std::fs::read_to_string(path) {
                    contract_source = Some(source);
                    break;
                }
            }

            let mut staking_pool_address = [0u8; 20];

            if let Some(source) = contract_source {
                if let Ok(compiled) = thunder_lang::compile_source(&source) {
                    let bytecode = bincode::serialize(&compiled).unwrap();
                    let mut deploy_tx = Transaction::new_deploy(
                        1, 
                        0, // Nonce 0
                        validator_addr, // Deployed by genesis validator
                        bytecode, 
                        50000, 
                        1
                    );
                    deploy_tx.sign(&self.key_pair); 
                    self.chain[0].transactions.push(deploy_tx.clone());
                    
                    staking_pool_address = state.derive_contract_address(&validator_addr, 0);
                    
                    if let Err(e) = state.apply_transaction(&deploy_tx) {
                        tracing::error!("Genesis deploy error: {:?}", e);
                    } else {
                        // Successfully deployed, register it in system_contracts
                        state.system_contracts.insert("StakingPool".to_string(), staking_pool_address);
                    }
                } else {
                    tracing::warn!("Failed to compile System Staking Contract at genesis");
                }
            } else {
                tracing::warn!("System Staking Contract source not found");
            }

            // 2. Genesis Stake as a ContractCall to 'deposit'
            let mut genesis_tx = Transaction::new_call(
                1,
                1, // User's second nonce (after deploy)
                validator_addr,
                staking_pool_address,
                stake,
                b"deposit".to_vec(),
                50000,
                1,
            );
            // Note: We also change its kind to `TransactionKind::Stake` internally so the consensus layer 
            // can easily recognize it without parsing call_data, but it remains a proper ContractCall on-chain!
            genesis_tx.kind = thunder_core::transaction::TransactionKind::Stake;
            genesis_tx.sign(&self.key_pair);
            
            self.chain[0].transactions.push(genesis_tx.clone());
            if let Err(e) = state.apply_transaction(&genesis_tx) {
                tracing::error!("Genesis stake error: {:?}", e);
            }
            
            let _ = state.commit();
        }

        Ok(())
    }

    /// Create a new consensus event from pending transactions.
    pub fn create_event(&mut self) -> Result<Event, String> {
        // Gather transaction hashes from the mempool.
        let tx_hashes: Vec<_> = self.mempool.iter().map(|tx| tx.hash()).collect();

        // Find self-parent (latest event by this node).
        let self_parent = self
            .consensus
            .dag
            .latest_event(&self.key_pair.address())
            .map(|e| e.hash)
            .unwrap_or([0u8; 32]);

        // Find other-parent (pick a random tip from another validator).
        let other_parent = self
            .consensus
            .dag
            .tips()
            .iter()
            .find(|h| {
                self.consensus
                    .dag
                    .get(h)
                    .map(|e| e.creator != self.key_pair.address())
                    .unwrap_or(false)
            })
            .copied()
            .unwrap_or([0u8; 32]);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut event = Event::new(
            self.key_pair.address(),
            self_parent,
            other_parent,
            tx_hashes,
            timestamp,
        );

        // Compute hash and sign.
        event.hash = crypto::hash_sha256(&event.signable_bytes());
        event.signature = self.key_pair.sign(&event.hash);

        // Insert into consensus DAG.
        self.consensus
            .add_event(event.clone())
            .map_err(|e| e.to_string())?;

        Ok(event)
    }

    /// Run one round of consensus and produce a block if possible.
    pub fn try_produce_block(&mut self) -> Option<Block> {
        let output = self.consensus.process()?;

        // Apply ordered transactions to the state.
        let mut block_txs = Vec::new();
        for tx_hash in &output.ordered_tx_hashes {
            if let Some(pos) = self.mempool.iter().position(|t| t.hash() == *tx_hash) {
                let tx = self.mempool.remove(pos);
                
                // Process staking directly before applying state
                if tx.kind == thunder_core::transaction::TransactionKind::Stake {
                    let duration = if tx.data.len() == 4 {
                        let mut bytes = [0u8; 4];
                        bytes.copy_from_slice(&tx.data);
                        u32::from_le_bytes(bytes)
                    } else {
                        0
                    };
                    if self.validator_set.get(&tx.from).is_some() {
                        let _ = self.validator_set.add_stake(&tx.from, tx.value);
                    } else {
                        let _ = self.validator_set.register(tx.from, tx.public_key, tx.value, duration);
                    }
                } else if tx.kind == thunder_core::transaction::TransactionKind::Unstake {
                    let _ = self.validator_set.unregister(&tx.from);
                }
                
                let _ = self.state.write().unwrap().apply_transaction(&tx);
                block_txs.push(tx);
            }
        }

        if block_txs.is_empty() {
            return None;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let prev_block = self.latest_block();

        // --- Dynamic Tokenomics (EIP-1559) ---
        let mut base_fee = 1; // 1 Gwei min
        let mut total_gas_burned = 0;

        // Network Congestion / aBFT Capacity Curve (PoS / High-TPS optimized)
        // Instead of PoW time-targets, aBFT fees surge ONLY if the block hits its computational byte limits.
        let target_capacity = 1000; // Expected average comfortable txs per batch

        if block_txs.len() as u64 > target_capacity {
            // High Congestion Penalty Surge (Block capacity exceeded normal bounds)
            let overflow = block_txs.len() as u64 - target_capacity;
            base_fee += overflow * 2;
        } else if block_txs.len() < 100 {
            // Cool Down Low Network Traffic
            base_fee = 1;
        }

        // Tally Total Burnt Gas
        for tx in &block_txs {
            total_gas_burned += tx.max_fee();
        }

        let combined_pool = total_gas_burned + 10_000_000; // Total Burnt Gas + 0.01 THDR Micro-Subsidy

        // Deflationary 50/50 Protocol Split
        let validator_reward = combined_pool / 2; // 50% Minted strictly to Validator
                                                  // The remaining 50% is Cryptographically BURNED (Never minted into existence)

        // -------------------------------------------------------------
        // DPoS Yield Splitting (Yield Farming / Retail Validation)
        // -------------------------------------------------------------
        let mut val_account = self
            .state
            .read()
            .unwrap()
            .get_account(&self.key_pair.address());

        let commission = (validator_reward * 5) / 100; // 5% Node Server Commission
        let delegator_pool = validator_reward - commission; // 95% Pro-rata Fractional Yield

        // 3 Dummy Retail Delegators (A = 60%, B = 30%, C = 10%)
        let addr_a = [0xAA; 20];
        let addr_b = [0xBB; 20];
        let addr_c = [0xCC; 20];

        let payout_a = (delegator_pool * 60) / 100;
        let payout_b = (delegator_pool * 30) / 100;
        let payout_c = delegator_pool - payout_a - payout_b;

        {
            let mut state = self.state.write().unwrap();

            // 1. Validator purely sucks up Commission fees
            val_account.balance = val_account.balance.saturating_add(commission);
            state.set_account(&self.key_pair.address(), val_account);

            // 2. Retail Delegators receive their passive yield fractions!
            let mut acc_a = state.get_account(&addr_a);
            acc_a.balance = acc_a.balance.saturating_add(payout_a);
            state.set_account(&addr_a, acc_a);

            let mut acc_b = state.get_account(&addr_b);
            acc_b.balance = acc_b.balance.saturating_add(payout_b);
            state.set_account(&addr_b, acc_b);

            let mut acc_c = state.get_account(&addr_c);
            acc_c.balance = acc_c.balance.saturating_add(payout_c);
            state.set_account(&addr_c, acc_c);
        }

        let state_root = self.state.read().unwrap().compute_state_root();

        let mut block = Block::new(
            prev_block,
            block_txs,
            state_root,
            self.key_pair.address(),
            self.key_pair.public_key(),
            timestamp,
            base_fee,
            validator_reward, // Track ONLY the actual tokens injected into the supply
        );
        block.sign(&self.key_pair);

        // Commit state changes and append block.
        self.state.write().unwrap().commit();
        self.chain.push(block.clone());

        Some(block)
    }

    /// Get a block by height.
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.chain.get(height as usize)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use thunder_core::state::Account;

    fn temp_node() -> Node {
        let kp = KeyPair::generate();
        let dir = std::env::temp_dir().join(format!("thunder_node_{}", rand::random::<u64>()));
        let config = NodeConfig {
            data_dir: dir.to_str().unwrap().to_string(),
            ..Default::default()
        };
        Node::new(kp, config)
    }

    #[test]
    fn test_node_genesis() {
        let node = temp_node();
        assert_eq!(node.height(), 0);
        assert_eq!(node.chain.len(), 1);
    }

    #[test]
    fn test_register_validator() {
        let mut node = temp_node();
        node.register_as_validator(10_000).unwrap();
        assert_eq!(node.validator_set.active_count(), 1);
    }

    #[test]
    fn test_add_transaction() {
        let mut node = temp_node();
        let sender = KeyPair::generate();
        let recipient = KeyPair::generate();

        // Give the sender some coins.
        node.state
            .write()
            .unwrap()
            .set_account(&sender.address(), Account::with_balance(1_000_000));

        let mut tx =
            Transaction::new_transfer(1, 0, sender.address(), recipient.address(), 100, 21_000, 1);
        tx.sign(&sender);
        node.add_transaction(tx).unwrap();
        assert_eq!(node.mempool.len(), 1);
    }
}
