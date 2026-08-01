# Build Stage
FROM rust:slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake build-essential

# Copy workspace source code
COPY . .

# Build the release binary
RUN cargo build --release --workspace

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies for networking
RUN apt-get update && apt-get install -y ca-certificates ufw && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder
COPY --from=builder /app/target/release/thunder-cli /usr/local/bin/thunder-cli

# Set the entrypoint to the CLI node runner
ENTRYPOINT ["thunder-cli", "node", "start"]

# Expose ports (9000 for P2P, 8080 for JSON-RPC)
EXPOSE 9000 8080
