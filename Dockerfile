# Stage 1: Build the application
FROM rust:latest AS builder

WORKDIR /app

# Copy only necessary files for building
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the release binary
RUN cargo build --release

# Stage 2: Create a minimal image
FROM debian:bookworm-slim

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/blockchain-indexer .

# Run the application
CMD ["./blockchain-indexer"]
