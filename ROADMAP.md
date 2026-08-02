# 🗺️ Thunder Blockchain: Development Roadmap

This document outlines the strategic and technical phases intended for the future development and maturation of the **Thunder Blockchain** ecosystem.

## ✅ Phase 1: Core Foundation (v1.0.0)
- **Architecture**: 7-crate modular architecture in Rust.
- **Cryptography**: Ed25519 signatures, SHA-256 hashing, Merkle Trees.
- **Consensus**: DAG-based PoS aBFT implementation (Rounds, Fame, Order).
- **Execution**: Stack-based ThunderVM with strict gas metering.
- **Language**: ThunderScript compiler capable of mapping AST to ThunderVM bytecode.
- **Storage**: Persistent raw and state storage migrated to high-throughput parallelized `RocksDB` (`rust-rocksdb`), bypassing monolithic `.db` constraints via multithreaded concurrent column families.

---

## ✅ Phase 2: High Performance Architecture (v1.0.5)
- **RocksDB Migration**: Transitioned from LevelDB to parallelized RocksDB instances for ultra-low latency transaction throughput.
- **Multithreading Query Architectures**: Implemented strictly shared-ref state evaluations enabling non-blocking RPC connections.
- **Strict Security Audits**: Attained zero-warning compliance natively across `cargo clippy` and mathematically sealed `unwrap()` panics in Consensus logic.

---

## ✅ Phase 3: Cross-Chain Bridging & VM Capabilities (v1.1.0)
- **Linear Memory Sandbox**: Mapped dynamic Byte arrays to VM states enabling unlimited length verification footprints (`MLoad`, `MStore`).
- **Cryptographic Enclaves**: Deployed direct `Keccak256` hashing and `VerifySig` validation opcodes strictly inside the instruction lifecycle.
- **Oracle Validator Relayers**: Integrated the asynchronous `thunder-relayer` websocket daemon to seamlessly bridge Sepolia/Ethereum `Vault.sol` deposit events securely into the Thunder environment via JSON-RPC.

---

## ✅ Phase 4: Network Maturation & Testnet (v1.1.5)
- **Bootnode Infrastructure**: Implemented hardcoded bootnodes to allow dynamic peer discovery. 
- **Multi-Node Live Testnet**: Deployed a fully isolated 3-validator distributed topological matrix via Docker Compose to simulate high-latency conditions.
- **Caching Pipelines**: Eliminated heavy `rust:slim` CI/CD compilation caches via precision `.dockerignore` overrides.

---

## 🚀 UPCOMING: Phase 5: Ecosystem Tooling (v2.0.0)
*Focus: Creating the accessible infrastructure needed for developers and users to interact with Thunder Blockchain.*

- **[ ] ThunderScan Block Explorer**: A web-based frontend.
- **[ ] Web3 SDK / JSON-RPC Client**: A JS/TS library to allow web applications to interact.
- **[ ] Browser Wallet Extension**: A non-custodial extension for key management.

---

## 🚀 UPCOMING: Phase 6: Post-Quantum Cryptography (v3.0.0)
- **[ ] CRYSTALS-Dilithium Migration**: Replace `Ed25519` elliptic curves with NIST `Dilithium2` algorithms.
- **[ ] PQC Gas Economy Adjustments**: Recalculate transaction gas costs natively.

---

## 🚀 UPCOMING: Phase 7: Extreme Scalability (v4.0.0)
- **[ ] Object-Centric Parallel Execution**: Process non-overlapping state transactions concurrently.
- **[ ] Dynamic State Sharding & Subnets**: Split the network into independent parallel shards.
- **[ ] ZK-Stateless Architecture**: Transition to zero-knowledge state-root validation.
