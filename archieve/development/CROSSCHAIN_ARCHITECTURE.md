# Thunder Blockchain: Cross-Chain Bridge Architecture 🌉

This document provides a deep technical architecture for the Thunder Cross-Chain Bridge, enabling bi-directional transfers of assets (e.g., BTC, ETH, SOL) between their native networks and Thunder Blockchain.

## 1. Core Architectural Components

The bridge operates on a **Lock & Mint / Burn & Unlock** paradigm. It consists of three primary layers:

### A. The External Vaults (Native Chains)
Smart contracts or custody solutions residing on external blockchains.
*   **Ethereum (EVM)**: A Solidity Smart Contract (`Vault.sol`) that allows users to deposit ETH or ERC-20 tokens. It locks the funds and emits a `Deposit` event.
*   **Solana**: A Rust-based Solana Program holding wrapped assets or native SOL in a Program Derived Address (PDA).
*   **Bitcoin**: Since BTC does not have smart contracts, a **Threshold Signature Scheme (TSS)** wallet utilizing Taproot is used as a decentralized custody vault.

### B. The Relayer Network (Oracle Nodes)
A decentralized network of off-chain Rust/Node.js validator nodes.
*   **Role**: They constantly listen to RPC nodes of both Thunder and external chains (e.g., Infura for ETH, self-hosted nodes for Thunder).
*   **Consensus (Multi-Sig)**: To prevent a single point of failure or malicious minting, the bridge requires a **2/3 supermajority** (e.g., 5 out of 7 Oracles) to sign off on any cross-chain event.

### C. Thunder Bridge Contract (Thunder Blockchain)
A native ThunderScript smart contract (`bridge.thunder`) that controls the Wrapped Assets (`tETH`, `tBTC`, `tSOL`).
*   It maintains a registry of authorized Relayer public keys.
*   It exposes a `verify_and_mint(payload, signatures)` function that requires cryptographically verifying the 2/3 threshold signature before minting tokens.

---

## 2. Transaction Lifecycles (Sequence Diagrams)

### 2.1 Inbound Diagram: Web3 to Thunder Blockchain (Lock & Mint)

```mermaid
sequenceDiagram
    participant User as User (ETH Wallet)
    participant EthVault as Ethereum Vault (Smart Contract)
    participant Relayers as Relayer Network (Oracles)
    participant Thunder as Thunder Bridge (Smart Contract)
    
    User->>EthVault: Deposit 10 ETH
    note over EthVault: ETH is locked in contract.
    EthVault->>EthVault: Emit Event: Locked(User, 10 ETH, ThunderAddress)
    
    Relayers-->>EthVault: (Listen) Detects Event
    note over Relayers: Wait for X Block Confirmations (Finality)
    Relayers->>Relayers: Sign payload (User, 10 ETH, TxHash)
    
    Relayers->>Thunder: Submit Multi-Sig Mint Payload
    note over Thunder: Verify 2/3 Signatures
    Thunder->>Thunder: Mint 10 tETH to User's Thunder Address
    Thunder-->>User: Balance Updated! ⚡
```

### 2.2 Outbound Diagram: Thunder to Web3 (Burn & Unlock)

```mermaid
sequenceDiagram
    participant User as User (Thunder Wallet)
    participant Thunder as Thunder Bridge (Smart Contract)
    participant Relayers as Relayer Network (Oracles)
    participant EthVault as Ethereum Vault (Smart Contract)
    
    User->>Thunder: Request Withdraw (Burn 10 tETH)
    note over Thunder: 10 tETH is destroyed from circulation.
    Thunder->>Thunder: Emit Event: Burn(User, 10 tETH, EthAddress)
    
    Relayers-->>Thunder: (Listen) Detects Event
    Relayers->>Relayers: Create & Sign EVM Unlock Payload
    
    Relayers->>EthVault: Call `unlock(EthAddress, 10 ETH, signatures)`
    note over EthVault: Verify Multi-Sig
    EthVault->>User: Transfer 10 ETH from Vault to User's ETH Wallet
```

---

## 3. High-Level Security & Risk Mitigation

Bridging holds massive systemic risk. The following architecture ensures funds cannot be drained:

1.  **Strict Rate Limiting (Circuit Breakers)**
    *   The `bridge.thunder` contract will enforce a maximum outflow per hour (e.g., 100 tETH per hour). If requested withdrawals exceed this, the bridge automatically pauses and alerts human admins.
2.  **Anti-Replay Protection**
    *   Every cross-chain transaction requires a unique **Nonce** or mapping of the originating `TxHash`. The Thunder contract will log processed hashes to ensure a Mint signature cannot be re-submitted twice.
3.  **TSS for Bitcoin Custody**
    *   Instead of a single custodian holding BTC, a decentralized Multi-Party Computation (MPC) scheme generates a Bitcoin address. No single Oracle holds the private key; they hold key shares that must be mathematically combined to release BTC.
4.  **Omnichain Restaking Authority Cap (15% Hard Limit)**
    *   To prevent external networks from hijacking consensus, Thunder implements a strict **Hybrid Sovereignty Cap**. Restaking from foreign chains (Ethereum, BSC, Solana) is permanently capped at a combined maximum of **15% total Voting Power / Fame**.
    *   At least **85%** of the network's consensus authority MUST strictly originate from Native Thunder Coin (TDR) Stakers. If foreign stakes exceed value limits due to Oracle price fluctuations, their voting weight is automatically mathematically diluted to maintain the 15% ceiling, preventing economic takeover via the Omnichain Bridge.

---

## 4. Omnichain Hub Defense-in-Depth Quadrant 🛡️

When bridging 3 massive independent external blockchains concurrently (Ethereum, BSC, and Solana), the attack vectors infinitely multiply. Thunder mitigates inter-chain catastrophe via 4 ultimate defense barriers:

### 4.1 Compartmentalized Fault Isolation (Submarine Silos)
If the Solana network suffers an external DDoS crash or blockchain halt, the Thunder Bridge does not freeze entirely. The network algorithm separates the voting delegations of BSC, SOL, and ETH into strict airtight sub-state silos. A failure in Solana purely freezes out the Solana delegation branch (restricting its max capped voting share), while native TDR, ETH, and BSC nodes continue propagating blocks smoothly without interruption.

### 4.2 Dual-Oracle TWAP Circuit Breaker
Since external native coins (ETH, BNB, SOL) are unified into standardized USD equivalence representation to determine Fair Voting Power, Price Oracles represent a severe vulnerability. Should a hacker manipulate Chainlink prices to spike Solana artificially, they could instantly monopolize Thunder's consensus power for pennies. 
**Defense**: The consensus aggregates Time-Weighted Average Prices (TWAP) spanning Chainlink, Pyth Network, and Band Protocol simultaneously. If deviance transcends >5%, the network instantly initiates an Automated Circuit Breaker—suspending Omnichain voting integrations until prices securely stabilize.

### 4.3 Native Light-Client State Proofs (Trustless Bridging)
The Thunder Virtual Machine eliminates HTTP Relayer Trust. Relayers do not assert that an event occurred; they merely transport the cryptographic payload. The `ThunderVM` natively interprets the raw `Block Headers` and `Merkle Patricia Trie Proofs` of Ethereum or Solana mathematically on-chain without trusting any middleman servers—replicating advanced Cosmos IBC topology.

### 4.4 Automated Multi-Chain Slashing Freezes
All foreign Restakers are hardcoded subject to a **7 to 21-Days Slashing Freeze Window**. Should an interacting Solana Restaker behave maliciously inside Thunder (e.g., executing a double spend), Thunder mathematically generates a `Slashing Proof`. Because their original Solana capital remains bound by the 21-day withdrawal lock, Relayers are granted enormous latency to submit this cryptographic verdict to the Solana Vault Program to autonomously burn their primary assets into oblivion.
