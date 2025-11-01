FROM rust:1.90.0 AS builder
WORKDIR /usr/src/noadick
COPY . .
RUN apt -y update && apt -y install openssl
RUN cargo install --path .

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/noadick /usr/local/bin/noadick
COPY --from=builder /usr/local/cargo/bin/migrate_json_to_bin /usr/local/bin/migrate_json_to_bin

COPY .release.env* ./.release.env
COPY .debug.env* ./.debug.env

ENV STORAGE_PATH=/app/storage

CMD ["noadick"]
