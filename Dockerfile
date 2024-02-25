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

FROM node:23 AS frontend
WORKDIR /app
COPY ui/harail/package.json ui/harail/package-lock.json ./
RUN npm install
COPY ui/harail/ .
RUN npm run build

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/harail_server /usr/local/bin
COPY --from=frontend /app/build /usr/share/harail/static

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080
ENTRYPOINT ["harail_server", "--static", "/usr/share/harail/static"]
