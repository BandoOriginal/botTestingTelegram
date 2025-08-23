FROM rust:nightly as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/rust-cron-job /usr/local/bin/app

CMD ["app"]
