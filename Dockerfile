# Build stage
FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/citoer /usr/local/bin/citoer
ENTRYPOINT ["citoer"]
