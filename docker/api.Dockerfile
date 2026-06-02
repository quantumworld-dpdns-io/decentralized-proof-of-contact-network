# syntax=docker/dockerfile:1.7
# Stage 1: Build Rust binaries
FROM rust:1.96-slim-bookworm AS builder

ARG TARGETARCH

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# Build release binaries
RUN cargo build --workspace --release && \
    ls -lh target/release/poi-api

# Stage 2: Runtime image
FROM debian:bookworm-slim

ARG TARGETARCH

# Runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    tini \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-privileged user
RUN groupadd -r poi && useradd -r -g poi -m -d /var/lib/poi -s /sbin/nologin poi

# Copy API binary
COPY --from=builder --chown=poi:poi /app/target/release/poi-api /usr/local/bin/

# Create directories
RUN mkdir -p /etc/poi /var/lib/poi && \
    chown -R poi:poi /etc/poi /var/lib/poi

# Copy config
COPY --chown=poi:poi config.prod.toml /etc/poi/config.toml

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:3000/health || exit 1

USER poi

EXPOSE 3000 9091

ENTRYPOINT ["/usr/bin/tini", "--", "poi-api"]

LABEL org.opencontainers.image.title="POI API Server" \
      org.opencontainers.image.description="Decentralized Proof-of-Contact Network API Server" \
      org.opencontainers.image.source="https://github.com/quantumworld-dpdns-io/decentralized-proof-of-contact-network" \
      org.opencontainers.image.licenses="MIT"
