FROM rust:1 AS chef
RUN cargo install --locked cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin harail_server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/harail_server /usr/local/bin
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080
ENTRYPOINT ["harail_server"]
