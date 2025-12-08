# Build stage (Compile Rust app)
FROM rust:1.84.0 as builder

# Set workspace directory
WORKDIR /workspace

# Install required dependencies
RUN apt-get update && apt-get install -y --no-install-recommends lld clang \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

# Now copy the rest of the source code
COPY src ./src
COPY scripts ./scripts
COPY settings ./settings
COPY static ./static

# Build the actual app (this will reuse cached dependencies)
RUN cargo build --release --locked

# Deploy stage (Minimal final image)
FROM gcr.io/distroless/cc

# Set working directory
WORKDIR /workspace

# Copy necessary files from builder
COPY --from=builder /workspace/target/release/app .
COPY --from=builder /workspace/scripts/run ./run
COPY --from=builder /workspace/settings settings
COPY --from=builder /workspace/static static

# Expose port
EXPOSE 3000

# Set environment variables
ENV APP_PROFILE=dev
ENV RUST_LOG=info

# Run the application
CMD ["./app"]
