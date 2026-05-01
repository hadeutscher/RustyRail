# ── Stage 1: dependency cache (cargo-chef) ─────────────────────────────────
FROM lukemathwalker/cargo-chef:0.1.77-rust-1.95-alpine3.23 AS chef
WORKDIR /app

# ── Stage 2: generate recipe ────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: build ──────────────────────────────────────────────────────────
FROM chef AS builder

# Install system deps
RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev openssl-libs-static perl

# Install dioxus-cli (dx) for building the WASM frontend
RUN cargo install dioxus-cli

# Pre-build (cache) the server dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --features server --recipe-path recipe.json

# Copy source
COPY . .

# Build the Dioxus app
WORKDIR /app
RUN dx bundle --package harail-app --release --debug-symbols=false

# ── Stage 4: minimal runtime image ──────────────────────────────────────────
FROM alpine:3.23

COPY --from=builder /app/app/dist /opt/harail

# Set our port and make sure to listen for all connections
ENV PORT=8080
ENV IP=0.0.0.0

EXPOSE 8080

ENTRYPOINT ["/opt/harail/server"]
