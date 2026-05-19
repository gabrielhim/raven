FROM rust:1.92-bookworm AS builder
WORKDIR /usr/src/raven
RUN apt-get update && apt-get install -y libclang-dev \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libcurl4-openssl-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/raven /usr/local/bin/raven
CMD ["raven"]
