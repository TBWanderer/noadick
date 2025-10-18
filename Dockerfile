FROM rust:1.90.0

WORKDIR /usr/src/noadick
COPY . .

RUN apt -y update && apt -y install openssl

ARG NOADICK_BOT_MODE="RELEASE"
RUN cargo install --path .

CMD ["noadick"]
