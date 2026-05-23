# syntax=docker/dockerfile:1.7
# Stage 1: Build Rust binaries
FROM rust:1.80-slim-bookworm AS builder

ARG TARGETARCH

WORKDIR /app

# Install system dependencies for cross-compilation if needed
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies first (caching layer)
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# Build all workspace crates in release mode
RUN cargo build --workspace --release && \
    # Verify binaries exist
    ls -lh target/release/poi-node target/release/poi-api

# Stage 2: Runtime image
FROM debian:bookworm-slim

ARG TARGETARCH

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    tini \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create poi user for privilege separation
RUN groupadd -r poi && useradd -r -g poi -m -d /var/lib/poi -s /sbin/nologin poi

# Copy binaries from builder
COPY --from=builder --chown=poi:poi /app/target/release/poi-node /usr/local/bin/
COPY --from=builder --chown=poi:poi /app/target/release/poi-api /usr/local/bin/

# Create config and data directories
RUN mkdir -p /etc/poi /var/lib/poi && \
    chown -R poi:poi /etc/poi /var/lib/poi

# Copy production configuration
COPY --chown=poi:poi config.prod.toml /etc/poi/config.toml

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:3000/health || exit 1

# Switch to non-root user
USER poi

# Expose ports: HTTP API, P2P networking, Prometheus metrics
EXPOSE 3000 9090 9091

# Use tini as init for proper signal handling
ENTRYPOINT ["/usr/bin/tini", "--", "poi-node", "--config", "/etc/poi/config.toml"]

# Metadata labels
LABEL org.opencontainers.image.title="POI Node" \
      org.opencontainers.image.description="Decentralized Proof-of-Contact Network Node" \
      org.opencontainers.image.source="https://github.com/quantumworld-dpdns-io/decentralized-proof-of-contact-network" \
      org.opencontainers.image.licenses="MIT"
