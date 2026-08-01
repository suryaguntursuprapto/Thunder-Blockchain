# ⚡ Thunder Blockchain

A blockchain network built from scratch in **Rust**, featuring **Proof of Stake** consensus with **Asynchronous Byzantine Fault Tolerance (aBFT)**, **LevelDB** storage, and **ThunderScript** — a custom smart contract language.

📚 **[Read the Whitepaper](WHITEPAPER.md)** | 🗺️ **[View the Development Roadmap](ROADMAP.md)**

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     thunder-cli (Binary)                        │
│              CLI interface & node orchestration                 │
├─────────────┬──────────────┬──────────────┬─────────────────────┤
│ thunder-rpc │thunder-network│thunder-lang  │                    │
│  JSON-RPC   │  P2P / Node  │ ThunderScript│                    │
│   API       │  libp2p      │  Compiler    │                    │
├─────────────┴──────┬───────┴──────────────┤                    │
│                    │                      │                    │
│  thunder-consensus │    thunder-vm        │                    │
│  PoS + aBFT        │    Stack-based VM    │                    │
│  DAG Hashgraph     │    Gas metering      │                    │
├────────────────────┴──────────────────────┴────────────────────┤
│                       thunder-core                             │
│         Crypto • Blocks • Transactions • Merkle Tree           │
│               State • LevelDB Storage                          │
└────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Description |
|---|---|
| `thunder-core` | Core data structures, SHA-256/Ed25519 cryptography, Merkle tree, LevelDB storage |
| `thunder-consensus` | PoS validator management, DAG-based aBFT consensus (Hashgraph-inspired) |
| `thunder-network` | P2P networking, peer management, node lifecycle |
| `thunder-vm` | Stack-based virtual machine with 40+ opcodes and gas metering |
| `thunder-lang` | ThunderScript lexer, parser, AST, and bytecode compiler |
| `thunder-rpc` | JSON-RPC 2.0 API server |
| `thunder-cli` | Command-line interface (main binary) |

## Quick Start

```bash
# Build the entire project
cargo build --workspace

# Run all tests
cargo test --workspace

# Create a wallet
cargo run -p thunder-cli -- wallet create

# Start a node
cargo run -p thunder-cli -- node start

# Compile a ThunderScript contract
cargo run -p thunder-cli -- contract compile examples/token.thunder

# Run a contract function locally
cargo run -p thunder-cli -- contract run examples/token.thunder -f init
```

## ThunderScript

ThunderScript is a custom smart contract language designed specifically for Thunder Blockchain.

```
contract Token {
    state owner: address;
    state balances: map<address, u64>;

    fn init() {
        self.owner = caller();
        self.balances[caller()] = 1000000;
    }

    fn transfer(to: address, amount: u64) {
        let sender_bal = self.balances[caller()];
        require(sender_bal >= amount, "Insufficient balance");
        self.balances[caller()] = sender_bal - amount;
        self.balances[to] = self.balances[to] + amount;
        emit Transfer(caller(), to, amount);
    }

    fn balance_of(addr: address) -> u64 {
        return self.balances[addr];
    }
}
```

### Language Features

- **Contract declarations** with state variables
- **Types**: `u64`, `bool`, `address`, `string`, `map<K, V>`
- **Control flow**: `if`/`else`, `while` loops
- **Functions** with parameters and return types
- **Built-in functions**: `caller()`, `balance()`, `timestamp()`, `block_height()`, `hash()`
- **State access**: `self.field`, `self.map[key]`
- **Safety**: `require(condition, "message")` for assertions
- **Events**: `emit EventName(args...)` for logging

## Consensus: PoS aBFT

Thunder Blockchain uses a **Hashgraph-inspired** Asynchronous Byzantine Fault Tolerance consensus:

1. **DAG Construction**: Validators create events that reference two parents (self-parent + other-parent)
2. **Divide Rounds**: Events are assigned round numbers based on strong-seeing
3. **Decide Fame**: Witnesses are determined to be "famous" via virtual voting
4. **Find Order**: Famous witnesses establish a total ordering of transactions
5. **Block Production**: Ordered transactions are applied to state and packed into blocks

This achieves consensus **without explicit message passing** — using virtual voting on the DAG.

### Validator Requirements

- Minimum stake: 1,000 THDR
- Supermajority: ⅔ of total stake
- Slashing for double-signing and inactivity

## License

MIT
