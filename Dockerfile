FROM rust:1.89.0 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
# Reemplaza telegram_auto_poster si tu bin se llama distinto
COPY --from=builder /out/bin/telegram_auto_poster /usr/local/bin/app

CMD ["app"]
