FROM rust:latest as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cd server && cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/harail_server /usr/local/bin/harail_server
ENTRYPOINT ["harail_server"]