FROM lukemathwalker/cargo-chef:0.1.71-rust-1.86-alpine3.21 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin harail_server

FROM node:23-alpine3.21 AS frontend
WORKDIR /app
COPY ui/package.json ui/package-lock.json ./
RUN npm install
COPY ui/ .
RUN npm run build

FROM alpine:3.21
COPY --from=builder /app/target/release/harail_server /usr/local/bin
COPY --from=frontend /app/dist /usr/share/harail/static

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080
ENTRYPOINT ["harail_server", "--static", "/usr/share/harail/static"]
