# syntax=docker/dockerfile:1.4.0
FROM --platform=linux/amd64 rust:1.69.0-slim-bullseye as builder

RUN apt-get update \
    && apt-get -y install \
    clang \
    cmake \
    libudev-dev \
    make \
    unzip \
    libssl-dev \
    pkg-config \
    libpq-dev \
    curl

RUN rustup component add rustfmt && update-ca-certificates

ENV HOME=/home/root
WORKDIR $HOME/app
COPY . .

# cache these directories for reuse
# see: https://docs.docker.com/build/cache/#use-the-dedicated-run-cache
RUN --mount=type=cache,mode=0777,target=/home/root/app/target \
    --mount=type=cache,mode=0777,target=/usr/local/cargo/registry \
    --mount=type=cache,mode=0777,target=/usr/local/cargo/git \
    cargo build --release --bin kamino-airdrop-api && cp target/release/kamino-airdrop-api ./

FROM --platform=linux/amd64 debian:bullseye-slim as base_image
RUN apt-get update && apt-get install -y libssl1.1 libpq-dev ca-certificates && update-ca-certificates && rm -rf /var/lib/apt/lists/*

FROM base_image as kamino-airdrop-api
WORKDIR /app
COPY --from=builder /home/root/app/kamino-airdrop-api ./
ENTRYPOINT ["./kamino-airdrop-api"]
