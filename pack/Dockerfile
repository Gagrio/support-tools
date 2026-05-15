FROM registry.suse.com/bci/rust:latest AS builder

WORKDIR /app

# Copy manifests first — dependencies built separately for layer caching
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build with real source
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM registry.suse.com/bci/bci-micro:latest

COPY --from=builder /app/target/release/pack /usr/local/bin/pack

RUN chmod 1777 /tmp

ENTRYPOINT ["/usr/local/bin/pack"]
