# ── Build stage ──────────────────────────────────────────────────
FROM rust:1.88-slim AS builder

WORKDIR /app

# Installer les dépendances système nécessaires à la compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copier les manifestes Cargo en premier pour profiter du cache Docker.
# Si seulement le code source change, les dépendances ne seront pas recompilées.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/core/Cargo.toml         crates/core/Cargo.toml
COPY crates/storage/Cargo.toml      crates/storage/Cargo.toml
COPY crates/network/Cargo.toml      crates/network/Cargo.toml
COPY crates/server/Cargo.toml       crates/server/Cargo.toml
COPY crates/audio/Cargo.toml        crates/audio/Cargo.toml
COPY crates/sprites/Cargo.toml      crates/sprites/Cargo.toml
COPY crates/tui/Cargo.toml          crates/tui/Cargo.toml
COPY crates/android/Cargo.toml      crates/android/Cargo.toml
COPY crates/ios/Cargo.toml          crates/ios/Cargo.toml
COPY crates/composer/Cargo.toml     crates/composer/Cargo.toml

# Créer des stubs vides pour que cargo puisse résoudre les dépendances
RUN mkdir -p crates/core/src crates/storage/src crates/network/src \
             crates/server/src crates/audio/src crates/sprites/src \
             crates/tui/src crates/android/src crates/ios/src crates/composer/src \
    && echo "fn main() {}" > crates/server/src/main.rs \
    && touch crates/core/src/lib.rs crates/storage/src/lib.rs \
             crates/network/src/lib.rs crates/audio/src/lib.rs \
             crates/sprites/src/lib.rs crates/tui/src/main.rs \
             crates/android/src/lib.rs crates/ios/src/lib.rs \
             crates/composer/src/main.rs

# Pré-compiler toutes les dépendances (sera caché si les Cargo.toml n'ont pas changé)
RUN cargo build --release --bin monster-battle-server 2>/dev/null || true

# Copier le vrai code source
COPY crates/ crates/

# Forcer la recompilation des crates locaux (les timestamps ont changé)
RUN touch crates/core/src/lib.rs \
          crates/storage/src/lib.rs \
          crates/network/src/lib.rs \
          crates/server/src/main.rs

# Build final avec Cargo.lock verrouillé
RUN cargo build --release --locked --bin monster-battle-server

# ── Runtime stage ────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/monster-battle-server /usr/local/bin/monster-battle-server

# Utilisateur non-root pour la sécurité
RUN useradd -r -s /bin/false appuser
USER appuser

ENV PORT=7878
EXPOSE 7878

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD sh -c 'echo >/dev/tcp/localhost/$PORT' || exit 1

CMD ["monster-battle-server"]
