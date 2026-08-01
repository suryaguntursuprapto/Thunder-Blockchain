---
description: Build and Run a Local Thunder Validator Node
---
# How to run your local Thunder Blockchain Node

Follow these steps to compile the Core Engine and deploy a single local node instance.

// turbo
1. Update crates and build the release binary
```bash
cargo build --workspace --release
```

// turbo
2. View the local node starting help menu
```bash
cargo run -p thunder-cli --release -- node --help
```

3. Start your local validator node on port 9000
```bash
cargo run -p thunder-cli --release -- node start --port 9000
```
