FROM rust:1.90.0 AS builder
WORKDIR /usr/src/noadick
COPY . .
RUN apt -y update && apt -y install openssl
RUN cargo build --release --bin migrate_json_to_bin
RUN cargo install --path .

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/noadick /usr/local/bin/noadick
COPY --from=builder /usr/src/noadick/target/release/migrate_json_to_bin /usr/local/bin/migrate_json_to_bin

COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["noadick"]
