FROM rust:latest as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cd server && cargo install --path .

FROM alpine:latest
COPY --from=builder /usr/local/cargo/bin/harail_server /usr/local/bin/harail_server
CMD ["harail_server"]