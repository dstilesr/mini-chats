FROM rust:1-alpine AS builder

WORKDIR /app

COPY rust-src/Cargo.toml .
COPY rust-src/Cargo.lock .
COPY rust-src/src ./src

RUN cargo build --release

FROM alpine:latest

ENV APP_PORT=3501
ENV APP_STATIC_PATH=/app/assets
EXPOSE 3501

WORKDIR /app

COPY assets ./assets
COPY --from=builder /app/target/release/minichat .

CMD ["./minichat"]
