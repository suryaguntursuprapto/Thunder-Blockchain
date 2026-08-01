# ⚡ Thunder Blockchain Whitepaper

## Abstract
Thunder Blockchain is a high-performance, modular blockchain protocol built from scratch in Rust. It aims to solve the scalability trilemma by leveraging an Asynchronous Byzantine Fault Tolerance (aBFT) consensus mechanism inspired by Hashgraph, coupled with a highly controlled, stack-based Virtual Machine (ThunderVM) and a domain-specific smart contract language (ThunderScript). This paper outlines the architecture, consensus rules, and economic model of the Thunder Blockchain.

---

## 1. Introduction
Traditional blockchains often struggle with throughput due to synchronous block production and heavy message-passing overhead during consensus. Thunder Blockchain removes the bottleneck of sequential block mining by utilizing a Directed Acyclic Graph (DAG) for event propagation. Validators communicate via a gossip protocol, continuously building a DAG of events that naturally achieves consensus through virtual voting.

## 2. Architecture
The Thunder Blockchain is logically divided into seven decoupled Rust crates to ensure modularity and maintainability:
- **`thunder-core`**: The foundational component handling cryptographic primitives (SHA-256 for hashing, Ed25519 for signatures), core data structures (`Transaction`, `Block`), Merkle trees, and persistent storage via LevelDB.
- **`thunder-consensus`**: Implements the aBFT consensus engine and Proof of Stake sybil resistance (discussed deeply in Section 3).
- **`thunder-network`**: A libp2p-based P2P networking layer handling peer discovery (Kademlia DHT) and transaction/event propagation (Gossipsub).
- **`thunder-vm`**: A stack-based deterministic virtual machine with strict gas metering.
- **`thunder-lang`**: The full compiler toolchain (Lexer, Parser, Compiler) for ThunderScript.
- **`thunder-rpc`**: JSON-RPC 2.0 API server for external interaction.
- **`thunder-cli`**: The unified command-line entry point for node operators and developers.

## 3. Consensus Mechanism: PoS aBFT
Thunder Blockchain employs a unique consensus model combining Proof of Stake (PoS) with Asynchronous Byzantine Fault Tolerance (aBFT).

### 3.1 The Event DAG
Instead of broadcasting blocks, validators broadcast `Events`. An Event contains:
- The creator's signature.
- A list of transaction payloads.
- A `self-parent` hash (the creator's previous event).
- An `other-parent` hash (an event received from another validator).
- A timestamp.

These events form a Directed Acyclic Graph (DAG) representing the history of gossiped information across the network.

### 3.2 Virtual Voting
Because the DAG records exactly *who* knew *what* and *when*, nodes do not need to send explicit voting messages to agree on the order of transactions. Thunder uses a three-step virtual voting algorithm:
1. **Divide Rounds**: Events are grouped into topological rounds based on "strong-seeing" (an event strongly sees another if there are multiple independent paths between them in the DAG).
2. **Decide Fame**: The first event created by a validator in a new round is a "witness". For each witness in round *R*, witnesses in round *R+1* cast virtual votes on whether it is "famous" (i.e., widely seen).
3. **Find Order**: Once fame is decided, events are sorted by the round they were received, their consensus timestamp, and their cryptographic hash to produce an immutable total order.

### 3.3 Sybil Resistance & Slashing
To prevent Sybil attacks, Thunder Blockchain uses Proof of Stake. Validators must lock a minimum amount of `THDR` tokens to participate. 
- Voting power is proportional to stake.
- Validators proven to have double-signed (created two conflicting events with the same self-parent) are immediately slashed and ejected from the validator set.

## 4. Smart Contracts: ThunderScript & ThunderVM
Execution on Thunder Blockchain is governed by the ThunderVM, optimized for security, determinism, and execution speed.

### 4.1 ThunderVM
ThunderVM is a stack-based virtual machine executing 64-bit opcodes. It features:
- **Strict Gas Metering**: Every opcode has a predefined gas cost (e.g., arithmetic costs ~3 gas, while persistent `SStore` costs 5,000 gas). Infinite loops result in an `Out of Gas` error, reverting all state changes.
- **Call Frames**: Supports deep functional execution and local variable state isolation.
- **Blockchain Context**: Immutable opcodes like `CALLER`, `BALANCE`, and `BLOCKHEIGHT` allow contracts to interact securely with the blockchain environment.

### 4.2 ThunderScript
To reduce the attack surface prevalent in Turing-complete languages like Solidity, Thunder Blockchain introduces `ThunderScript`.
- **C-Like Syntax**: Familiar to developers, lowering the barrier to entry.
- **Safety First**: Incorporates strict typing and mandatory `require` assertions.
- **State Maps**: Natively supports `map<K, V>` for highly efficient localized storage access.

## 5. Economic Model
(To be finalized in future phases) 
Thunder Blockchain will utilize a deflationary tokenomics model where a percentage of the base transaction fee (gas) is burned, while a priority fee is awarded to the block proposer. Inflation is strictly controlled via epoch-based staking rewards to incentivize network security.

## 6. Conclusion
Thunder Blockchain provides a mathematically rigorous, aBFT-secure foundation for a high-throughput decentralized ecosystem. Its modular Rust architecture and custom VM ensure it remains future-proof and resilient.
