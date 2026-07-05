# Fridgly 🧊

A dead-simple fridge tracker: see at a glance what you have and what's about to
expire, so you stop buying duplicates and throwing food away.

The interface is intentionally minimal — adding an item should take seconds, so
nobody in the household skips it.

## Status

Phase 1 (manual entry) — usable today:

- Add items with a name, quantity and expiry (one-tap quick-picks: Today / +3d /
  +1wk / +1mo, or pick a date).
- Home screen lists everything in the fridge, **sorted by expiry**, with colour
  bands (🔴 expired · 🟠 ≤2d · 🟡 this week · 🟢 later · ⚪ no date).
- Edit inline, mark **used** (✓) or **tossed** (🗑) — soft-removed so we can
  report on waste later.

Planned next: barcode scanning (camera → [Open Food Facts](https://world.openfoodfacts.org/)
lookup), common-items quick-add, then optional reminders.

## Architecture

A layered Cargo workspace (ports & adapters) so the codebase stays maintainable
as it grows and multiple people work on it:

| Crate                       | Responsibility                                             | Depends on            |
| --------------------------- | --------------------------------------------------------- | --------------------- |
| `crates/fridgly-domain`     | Business model & rules (`Item`, expiry/urgency), the `ItemRepository` **port**. No framework deps. | —                     |
| `crates/fridgly-infra`      | Postgres **adapter** for the port, connection pool, migrations. | `domain`, `sqlx`      |
| `crates/fridgly-web`        | Axum HTTP server, feature-sliced handlers, Askama + htmx UI. | `domain`, `infra`     |

The web layer depends on the repository *trait*, never on Postgres directly, so
storage is swappable and the domain is unit-testable in isolation.

```
crates/fridgly-web/src/
├── main.rs          # composition root: config → pool → migrate → serve
├── config.rs        # env-driven configuration
├── state.rs         # shared AppState (repository behind a trait object)
├── error.rs         # AppError → HTTP status mapping
├── app.rs           # router assembly
└── features/
    └── items/       # one feature slice: routes, forms, view models, handlers
```

## Tech stack

Rust · Axum · Postgres (via `sqlx`) · Askama templates · htmx.

## Getting started

Prerequisites: the pinned Rust toolchain (installed automatically via
[`rust-toolchain.toml`](rust-toolchain.toml)) and Docker.

```bash
# 1. Start Postgres
docker compose up -d

# 2. Configure environment
cp .env.example .env

# 3. Run (migrations apply automatically on startup)
cargo run

# App: http://localhost:3000
```

## Development

```bash
cargo test          # run unit tests (domain rules, etc.)
cargo fmt           # format
cargo clippy        # lint
make check          # fmt-check + clippy + test (used by CI)
```

Database migrations live in [`migrations/`](migrations/) and are embedded into
the binary at build time, so a release deployment needs only the binary + static
assets.
