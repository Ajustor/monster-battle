# --- Build stage ---
FROM rust:1.85-slim AS builder

WORKDIR /app

# Copier les manifestes d'abord pour profiter du cache Docker
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml crates/core/Cargo.toml
COPY crates/storage/Cargo.toml crates/storage/Cargo.toml
COPY crates/network/Cargo.toml crates/network/Cargo.toml
COPY crates/tui/Cargo.toml crates/tui/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml

# Créer des fichiers sources vides pour que cargo puisse résoudre les deps
RUN mkdir -p crates/core/src crates/storage/src crates/network/src crates/tui/src crates/server/src && \
  echo "fn main() {}" > crates/server/src/main.rs && \
  echo "fn main() {}" > crates/tui/src/main.rs && \
  touch crates/core/src/lib.rs crates/storage/src/lib.rs crates/network/src/lib.rs

# Pré-compiler les dépendances (cache)
RUN cargo build --release --bin monster-battle-server 2>/dev/null || true

# Copier le vrai code source
COPY crates/ crates/

# Forcer la recompilation des crates locaux
RUN touch crates/core/src/lib.rs crates/storage/src/lib.rs crates/network/src/lib.rs crates/server/src/main.rs

# Build final
RUN cargo build --release --bin monster-battle-server

# --- Runtime stage ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
  rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/monster-battle-server /usr/local/bin/monster-battle-server

ENV PORT=7878
ENV HEALTH_PORT=8080

EXPOSE 7878
EXPOSE 8080

CMD ["monster-battle-server"]
