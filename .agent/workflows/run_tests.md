---
description: Run the comprehensive test suite across the Thunder Workspace
---
# Run Workspace Test Suite

Follow these steps to ensure all cryptographic, network, and virtual machine implementations are verified and correct.

// turbo-all

1. Run the core cryptographic and storage tests
```bash
cargo test -p thunder-core
```

2. Run the Virtual Machine and bytecode tests
```bash
cargo test -p thunder-vm
```

3. Run the ThunderScript compiler tests
```bash
cargo test -p thunder-lang
```

4. Run all integrated workspace tests
```bash
cargo test --workspace
```
