# --- Build stage ---
FROM rust:1.88-slim AS builder

WORKDIR /app

# Copier les manifestes d'abord pour profiter du cache Docker
COPY . .

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

EXPOSE 7878

CMD ["monster-battle-server"]
