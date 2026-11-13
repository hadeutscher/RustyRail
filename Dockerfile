# ── Stage 1: dependency cache (cargo-chef) ─────────────────────────────────
FROM rust:1.100-trixie AS chef
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/v1.27.0/install-from-binstall-release.sh | BINSTALL_VERSION=1.27.0 bash
RUN cargo binstall --no-confirm --version 0.1.78 cargo-chef
WORKDIR /app

# ── Stage 2: generate recipe ────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: build ──────────────────────────────────────────────────────────
FROM chef AS builder

# Install dioxus-cli (dx)
RUN cargo binstall --no-confirm --version 0.7.10 dioxus-cli

# Pre-build (cache) the server dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --locked --release --features server --recipe-path recipe.json

# Copy source
COPY . .

# Build the Dioxus app
WORKDIR /app
RUN dx bundle --locked --package harail-app --release --debug-symbols=false

# ── Stage 4: minimal runtime image ──────────────────────────────────────────
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/app/dist /opt/harail

# Set our port and make sure to listen for all connections
ENV PORT=8080
ENV IP=0.0.0.0

EXPOSE 8080

ENTRYPOINT ["/opt/harail/server"]
