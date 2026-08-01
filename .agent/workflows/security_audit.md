---
description: Run the Security Audit & Code Linting suite for the Blockchain
---
# Continuous Security Auditing

In a blockchain environment, security is paramount. This workflow runs strict Rust linters and checks all of our dependencies against the official RustSec Advisory Database for any known vulnerabilities (CVEs).

// turbo-all

1. Run Cargo Clippy to enforce strict memory safety and code quality
```bash
cargo clippy --workspace -- -D warnings
```

2. Install cargo-audit if it is not already installed
```bash
cargo install cargo-audit
```

3. Run a deep security audit on all dependency crates
```bash
cargo audit
```
