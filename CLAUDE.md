# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Orapa Mine — web adaptation (2 players + solo training mode) of the tangram
deduction board game *Orapa Mine* (Playte 2024 / Miraludo 2026). Players
hide 5 tangram gems on a 10×8 grid; the opponent fires beams from the
perimeter and must deduce gem placement from exit point and beam color.
User-facing text, comments, commits, and the README are in French.

## Commands

```sh
# Rust: engine, server, wasm bindings, CLI demo
cargo test --workspace                        # all Rust tests (engine, server, wasm)
cargo test -p orapa-core                       # engine only (run after editing data/pieces.json)
cargo clippy --workspace --all-targets
cargo run -p orapa-cli -- --seed 1 --all       # ASCII trace of all 36 beams on a random board
cargo run -p orapa-cli -- --seed 1 --beam 7    # single beam
cargo run -p orapa-server                      # native server, listens on :8080

# Web client (run from web/)
npm install
npm run build:wasm   # (re)generate WASM bindings into src/wasm — required after any
                      # change to crates/orapa-core or crates/orapa-wasm
npm run dev           # Vite dev server on :5173, proxies /ws -> :8080
npm run build          # build:wasm + tsc -b + vite build
npx tsc -b && npx oxlint src   # typecheck + lint (no separate test suite on the client)
```

Local dev needs both the client and server running (client on 5173, WS to
server on 8080). For a production-like single-process check:
`npm run build` (in `web/`) then `ORAPA_STATIC_DIR=web/dist cargo run -p orapa-server`.

Docker: `docker compose up --build` — single multi-stage image serving
everything on `http://localhost:8080`, no database (games live in server
memory).

Env vars for the server: `PORT` (default 8080), `ORAPA_STATIC_DIR` (default
`web/dist`), `RUST_LOG`.

## Architecture

```
crates/
  orapa-core/    pure engine (geometry, beam tracing, colors, placement,
                 solution checking) — no UI/network dependency
  orapa-wasm/    wasm-bindgen bindings around orapa-core, for the client
  orapa-cli/     ASCII demo of the engine
  orapa-server/  axum server: per-game state machine, WebSocket protocol
data/
  pieces.json    piece catalogue (see below)
web/             React + TypeScript client (Vite)
```

`orapa-core` is the single implementation of geometry, beam tracing,
placement validation, and color mixing — used as-is both natively by the
server (Rust) and compiled to WebAssembly for the client. Secret gem
layouts are **never** sent to the opponent before game end; all beam
computation, probing, and solution verification happens server-side. The
client's WASM copy of the engine is used only for local hypothesis/help
mode, working off the move history the server sends — no opponent secret
data ever reaches it.

Because the engine is shared source (not just a shared protocol), a change
to beam/geometry/placement logic in `orapa-core` must be followed by
`npm run build:wasm` before the client build reflects it, and by
`cargo test -p orapa-core` to catch geometry regressions.

### Piece definitions (`data/pieces.json`)

Each piece is described only by its **base shape** (orientation 0): a list
of relative cells `{x, y, kind}`, where `kind` is `"square"` or one of
`"tri_nw"/"tri_ne"/"tri_se"/"tri_sw"` (a triangle occupying half the cell,
against that corner). All valid orientations (90° rotations, mirroring)
are generated and deduplicated automatically by the engine from this base
shape — do not hand-list them. To fix a piece's shape, edit only its
`cells` array, then run `cargo test -p orapa-core` (a test asserts every
generated orientation has the same area as the base shape, catching most
entry errors). The file also sets the grid dimensions (`grid`) and whether
the white-gem symmetry constraint is active
(`rules.white_symmetry_forbidden`).

### Beam model

"Cell-center" model: the beam travels center-to-center, entering/exiting
through the middle of cell edges — it never passes exactly through a grid
vertex, so exact-corner and edge-grazing cases cannot occur (guaranteed by
construction). Orthogonal face (square, a triangle's leg) → reflects back;
triangle hypotenuse → 90° deflection. Black body → absorbs the beam
(`Absorbed`). Closed loops (beam bouncing in a cycle between pieces) are
detected via repeated `(cell, direction)` state → `Lost`, with a max-step
safety net as backup. See
`crates/orapa-core/src/beam.rs::tests::closed_loop_between_pieces_is_detected_as_lost`.

### Known limitations

- Exact base-piece geometry from the official rulebook was unavailable
  during development; shapes in `data/pieces.json` are a best-effort
  reconstruction, correctable there if needed.
- Games live only in server process memory. Reconnection after page
  reload/network drop works (client-side token); a server restart drops
  all in-progress games.
