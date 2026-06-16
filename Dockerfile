# Build stage
FROM rust:latest AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

# Final stage
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/citoer /usr/local/bin/citoer
ENTRYPOINT ["citoer"]
