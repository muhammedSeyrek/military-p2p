FROM rust:1-bookworm

WORKDIR /build

# Copy cargo manifests (for dependency cache)
COPY Cargo.toml Cargo.lock ./
COPY crates/mp-crypto/Cargo.toml         crates/mp-crypto/Cargo.toml
COPY crates/mp-protocol/Cargo.toml       crates/mp-protocol/Cargo.toml
COPY crates/mp-storage/Cargo.toml        crates/mp-storage/Cargo.toml
COPY crates/mp-network/Cargo.toml        crates/mp-network/Cargo.toml
COPY crates/mp-node-general/Cargo.toml   crates/mp-node-general/Cargo.toml
COPY crates/mp-node-commander/Cargo.toml crates/mp-node-commander/Cargo.toml

# Create empty resources → compile only dependencies (cache layer)
RUN mkdir -p crates/mp-crypto/src crates/mp-protocol/src crates/mp-storage/src \
    crates/mp-network/src crates/mp-node-general/src crates/mp-node-commander/src && \
    echo "fn main() {}" > crates/mp-node-general/src/main.rs && \
    echo "fn main() {}" > crates/mp-node-commander/src/main.rs && \
    echo "" > crates/mp-crypto/src/lib.rs && \
    echo "" > crates/mp-protocol/src/lib.rs && \
    echo "" > crates/mp-storage/src/lib.rs && \
    echo "" > crates/mp-network/src/lib.rs && \
    mkdir -p crates/mp-storage/migrations/general crates/mp-storage/migrations/commander && \
    cargo build --release --bin mp-node-general --bin mp-node-commander 2>/dev/null || true

# Copy the actual source code.
COPY crates ./crates
COPY data   ./data

# The original build
RUN touch crates/mp-storage/src/lib.rs && \
    cargo build --release --bin mp-node-general --bin mp-node-commander && \
    cp target/release/mp-node-general   /usr/local/bin/ && \
    cp target/release/mp-node-commander /usr/local/bin/

WORKDIR /app
RUN cp -r /build/crates/mp-storage/migrations . && \
    cp -r /build/data .

CMD ["mp-node-general", "--help"]