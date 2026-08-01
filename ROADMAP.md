# 🗺️ Thunder Blockchain: Future Development Roadmap

This document outlines the strategic and technical phases intended for the future development and maturation of the **Thunder Blockchain** ecosystem.

## Phase 1: Core Foundation (✅ Complete)
- **Architecture**: 7-crate modular architecture in Rust.
- **Cryptography**: Ed25519 signatures, SHA-256 hashing, Merkle Trees.
- **Consensus**: DAG-based PoS aBFT implementation (Rounds, Fame, Order).
- **Execution**: Stack-based ThunderVM with strict gas metering.
- **Language**: ThunderScript compiler capable of mapping AST to ThunderVM bytecode.
- **Storage**: Persistent raw and state storage via LevelDB.

---

## Phase 2: Network Maturation & Testnet
*Focus: Stabilizing the peer-to-peer layer and ensuring consensus safety in a distributed environment.*

- **[ ] Bootnode Infrastructure**: Implement hardcoded bootnodes to allow dynamic peer discovery beyond Localhost.
- **[ ] Multi-Node Live Testnet**: Deploy a 5-validator local testnet utilizing Docker Compose to simulate high-latency conditions.
- **[ ] Sync Protocol Optimization**: Improve block and DAG event syncing for new nodes joining the network.
- **[ ] Metrics & Observability**: Expose Prometheus metrics for P2P latency, DAG round velocity, and gas consumption.

---

## Phase 3: Ecosystem Tooling
*Focus: Creating the accessible infrastructure needed for developers and users to interact with Thunder Blockchain.*

- **[ ] ThunderScan Block Explorer**: A web-based frontend (Next.js/React) connecting to `thunder-rpc` to visualize blocks, transactions, and DAG events in real-time.
- **[ ] Web3 SDK / JSON-RPC Client**: A JS/TS library to allow web applications to easily sign transactions and call ThunderScript contracts.
- **[ ] Browser Wallet Extension**: A non-custodial extension for key management and transaction signing (similar to MetaMask).

---

## Phase 4: Post-Quantum Cryptography (PQC) Upgrade
*Focus: Long-term security and resilience against Quantum Computing algorithms like Shor's.*

- **[ ] CRYSTALS-Dilithium Migration**: Replace `Ed25519` elliptic curve signatures with the NIST-standardized `Dilithium2` lattice-based signature scheme.
- **[ ] State Storage Optimization**: Repackaging LevelDB serializations (via variable binary vectors instead of `BigArray`) to accommodate the drastically larger Dilithium key/signature footprints.
- **[ ] PQC Gas Economy Adjustments**: Recalculate transaction gas costs mathematically tied to the exponential byte size of lattice cryptographic verification compared to classic cryptography.

---

## Phase 5: Advanced Smart Contracts (ThunderScript v2)
*Focus: Expanding the capabilities of the smart contract language.*

- **[ ] Advanced Data Types**: Add support for `struct`, arrays, and nested maps.
- **[ ] Cross-Contract Calls**: Allow smart contracts to securely instantiate and call functions in other smart contracts.
- **[ ] Standard Library**: Introduce built-in mathematical and cryptographic functions (e.g., `secp256k1` verification) to the compiler.
- **[ ] ABI Generation**: Output standard ABI files during compilation for easier frontend integration.

---

## Phase 6: Economic Finality & Tokenomics
*Focus: Enforcing the monetary policy and economic security of the network.*

- **[ ] Staking Rewards**: Implement algorithmic token issuance and distribution to validators based on uptime and stake weight.
- **[ ] Slashing Implementation**: Enforce the burning of stake for malicious behavior (double-signing events).
- **[ ] EIP-1559 Style Fee Market**: Implement a dynamic base fee that burns THDR to counteract inflation.

---

## Phase 7: Mainnet Launch
*Focus: Security audits, bug bounties, and the production genesis block.*

- **[ ] Security Audits**: Third-party review of the `thunder-consensus` aBFT logic and `thunder-vm` execution loop.
- **[ ] Genesis Ceremony**: Distribution of initial stake, locking the genesis block.
- **[ ] Mainnet Alpha (Restricted Validator Set)**: Launching with a trusted set of validators before slowly transitioning to a permissionless PoS network.
