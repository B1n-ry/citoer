# Build stage
FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/citoer /app/citoer
ENTRYPOINT ["/app/citoer"]
