# ── Stage 1: dependency cache (cargo-chef) ─────────────────────────────────
FROM lukemathwalker/cargo-chef:0.1.77-rust-1.95-trixie AS chef
WORKDIR /app

# ── Stage 2: generate recipe ────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: build ──────────────────────────────────────────────────────────
FROM chef AS builder

# Install dioxus-cli (dx)
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli

# Pre-build (cache) the server dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --features server --recipe-path recipe.json

# Copy source
COPY . .

# Build the Dioxus app
WORKDIR /app
RUN dx bundle --package harail-app --release --debug-symbols=false

# ── Stage 4: minimal runtime image ──────────────────────────────────────────
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/app/dist /opt/harail

# Set our port and make sure to listen for all connections
ENV PORT=8080
ENV IP=0.0.0.0

EXPOSE 8080

ENTRYPOINT ["/opt/harail/server"]
