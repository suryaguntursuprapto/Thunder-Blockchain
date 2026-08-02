<p align="center">
  <img src="assets/logo.png" width="300" alt="Thunder Blockchain Logo">
</p>

# 🚀 Thunder Blockchain - High-Performance Decentralized Network
![License: BSL 1.1](https://img.shields.io/badge/License-BSL%201.1-blue.svg) ![Rust](https://img.shields.io/badge/Rust-Production-orange.svg)

## 🌟 Overview

Thunder is a high-performance, modular blockchain network built entirely in Rust. It is designed for maximum throughput, leveraging a DAG-based **Virtual Voting (aBFT)** consensus algorithm (Hashgraph-inspired), and features a custom virtual machine (**ThunderVM**) powered by its own deeply-integrated language (**ThunderScript**).

## 📜 Licensing & Architecture Structure

This project uses dual licensing and is being structured into the following modular components for future scalability:

### 🔐 Blockchain Infrastructure (BSL 1.1)

**Components (Future Architecture):**
- `Core-Engine/` - Consensus (aBFT), mempool, state management, sharding
- `development/` - Node implementation, integration, contracts, ThunderVM
- `audit/` - Security audit tools and compliance tests
- `governance/` - DAO and governance mechanisms
- `deployment/` - Node deployment and orchestration
- `infrastructure/` - RPC API, node infrastructure, backend services
- `testing/` - Integration tests, testnet tools
- `monitoring/` - Production monitoring and metrics

**License:** Business Source License 1.1 - Perpetual Proprietary
**Usage:**
- ✅ Non-production use (testing, development, evaluation) - FREE
- ⚠️ Production use requires commercial license
- 🔒 Proprietary license - remains under BSL 1.1 indefinitely

### 📱 Client Applications (Apache-2.0)

**Components:**
- `applications/thunder-mobile/` - Mobile wallet
- `applications/thunder-wallet/` - Browser extension wallet
- `applications/thunderscan/` - Blockchain explorer
- `applications/thunder-cli/` - Command-line tools (currently in `Core-Engine`)

**License:** Apache License 2.0
**Usage:** Fully open-source - use, modify, distribute freely

---

## 🚀 CURRENT FEATURES (v1.0.1)

### ⚡ ThunderVM & ThunderScript
- **Stack-based VM**: Extremely lightweight execution environment.
- **Micro-Gas Metering**: Every single opcode executed consumes precise gas to prevent infinite loops and bounded computation.
- **Custom Compiler**: The `thunder-lang` crate contains a full Lexer, Parser, AST Generator, and Compiler that translates ThunderScript directly to Bytecode.

### 🏛️ Directed Acyclic Graph (DAG) aBFT Consensus
- **Virtual Voting**: Nodes do not need to send voting messages across the network. By gossiping "Events" containing 2 hashes (Self-Parent & Other-Parent), every node can mathematically calculate what everyone else would vote for.
- **Leaderless**: No block producers or leaders to attack. All nodes produce events continually.
- **Absolute Finality**: Transactions reach 100% mathematical finality without probabilistic rollbacks.

### 🔐 Cryptography & Storage
- **Ed25519 Signatures**: Ultra-fast signature verification using `ed25519-dalek`.
- **SHA-256 State Roots**: Strong 256-bit cryptographic Merkle Trees for state validation.
- **LevelDB State**: High-performance persistent raw and state storage (`rusty-leveldb`).

### 🌐 Peer-to-Peer Network
- **LibP2P Integration**: Secure and modular networking via `libp2p`.
- **GossipSub Integration**: Decentralized event propagation with mesh networking.
- **Kademlia DHT**: Node discovery without centralized bootstrap servers.

---

## ⚠️ UPCOMING UPGRADES (ROADMAP SUMMARY)

- **Phase 2: Network Maturation (v2.0.0)**
  Implementation of bootnode infrastructure, and syncing protocols for a multi-node docker testnet.
  
- **Phase 3: Ecosystem Tooling (v3.0.0)**
  Launch of `ThunderScan` Block Explorer, Browser Wallet Extensions, and comprehensive Web3 SDK for JS/TS frontends.

- **Phase 4: Post-Quantum Cryptography (v4.0.0)**
  Migrating from Ed25519 to `CRYSTALS-Dilithium` lattice-based signatures to provide mathematical resistance against future quantum computer attacks (e.g., Shor's Algorithm).
  
- **Phase 5: Extreme Scalability Architecture (v5.0.0)**
  Overhauling ThunderVM for **Object-Centric Parallel Execution** capable of scaling up to 400.000+ TPS via Dynamic State Sharding & ZK-Stateless architecture.

- **Phase 6: Advanced Smart Contracts (v6.0.0)**
  Adding support for custom structs, arrays, cross-contract calls, and standard ABI generation.

- **Phase 7: Economic Finality & Tokenomics (v7.0.0)**
  Implementation of PoS Staking algorithms, malicious behavior Slashing conditions, and an EIP-1559 base fee burning mechanism.

- **Phase 8: Mainnet Launch (v8.0.0)**
  Third-party security audits, bug bounties, and locking the production Genesis Block.

---

## �️ QUICK START

### Prerequisites
- **Rust**: Edition 2021 (v1.70+)
- **OS**: Linux, macOS, or WSL2

### Building the Project
```bash
git clone https://github.com/suryaguntursuprapto/Thunder-Blockchain.git
cd Thunder-Blockchain
cargo build --release
```

### Running the Node
```bash
# Start a node with the CLI
cargo run -p thunder-cli -- node start --port 9000

# Start a JSON-RPC Server
cargo run -p thunder-rpc
```

---

## 📚 API REFERENCE (JSON-RPC 2.0)

Nodes expose a JSON-RPC endpoint at `http://127.0.0.1:8080`.

**Check Balance:**
```json
{
  "jsonrpc": "2.0",
  "method": "get_balance",
  "params": ["<address>"],
  "id": 1
}
```

**Send Transaction:**
```json
{
  "jsonrpc": "2.0",
  "method": "send_transaction",
  "params": [{
    "sender": "<address>",
    "recipient": "<address>",
    "amount": 1000,
    "nonce": 1,
    "signature": "<hex_signature>"
  }],
  "id": 2
}
```

---

⚠️ **Disclaimer:** Thunder Blockchain is experimental software currently in the Testnet phase. Use at your own risk. Always test thoroughly before securing real assets.
