# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Étape 1 : compile le moteur en WebAssembly pour le client.
# ---------------------------------------------------------------------------
FROM rust:1-slim-bookworm AS wasm-builder
RUN rustup target add wasm32-unknown-unknown \
    # Version figée sur celle de la dépendance `wasm-bindgen` du workspace
    # (voir Cargo.lock) : les deux DOIVENT rester synchronisées, sous peine
    # d'échec au chargement du module côté navigateur.
    && cargo install wasm-bindgen-cli --version 0.2.127 --locked
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY data ./data
COPY crates ./crates
RUN cargo build --release --target wasm32-unknown-unknown -p orapa-wasm \
    && wasm-bindgen target/wasm32-unknown-unknown/release/orapa_wasm.wasm \
        --out-dir /app/wasm-out --target web --typescript

# ---------------------------------------------------------------------------
# Étape 2 : build du client React (Vite).
# ---------------------------------------------------------------------------
FROM node:22-slim AS web-builder
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web ./
COPY --from=wasm-builder /app/wasm-out ./src/wasm
# `build:wasm` est sauté ici (déjà fait à l'étape 1, pas de toolchain Rust
# dans cette image) : on enchaîne directement tsc + vite build.
RUN npx tsc -b && npx vite build

# ---------------------------------------------------------------------------
# Étape 3 : compile le serveur natif (release).
# ---------------------------------------------------------------------------
FROM rust:1-slim-bookworm AS server-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY data ./data
COPY crates ./crates
RUN cargo build --release -p orapa-server

# ---------------------------------------------------------------------------
# Étape 4 : image d'exécution minimale.
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /app orapa
WORKDIR /app
COPY --from=server-builder /app/target/release/orapa-server ./orapa-server
COPY --from=web-builder /app/web/dist ./web/dist
USER orapa
ENV ORAPA_STATIC_DIR=/app/web/dist
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["./orapa-server"]
