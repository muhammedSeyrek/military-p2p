FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-client \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

RUN mkdir -p /usr/local/cargo/registry /workspace/target \
    && chown -R vscode:vscode /usr/local/cargo /workspace

USER vscode

RUN cargo install sqlx-cli --no-default-features --features postgres --locked

WORKDIR /workspace