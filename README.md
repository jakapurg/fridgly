# Fridgly 🧊

A dead-simple fridge tracker: see at a glance what you have and what's about to
expire, so you stop buying duplicates and throwing food away.

The interface is intentionally minimal — adding an item should take seconds, so
nobody in the household skips it. Mobile-first, built to the
["Fridgly Wireframes" design](#design) (option **2a** — compact rows with a left
colour band and a bottom tab bar).

## Features

**Fridge (functional today)**

- **Add** via a bottom sheet: name, a quantity stepper, and one-tap expiry chips
  (Today / +3d / +1wk / +1mo, or pick a date).
- **Home list** sorted by expiry with colour bands: 🔴 expired · 🟠 ≤2d ·
  🟡 this week · 🟢 later · ⚪ no date. A legend explains them.
- **Search bar** — live, debounced filtering by name (case-insensitive), backed
  by a SQL `ILIKE` query so it stays fast with 30+ items. Distinct "no matches"
  state.
- **Edit** inline (tap a row); mark **used** (✓) or **tossed** (🗑) — soft-removed
  (status kept) so we can report on waste later.
- **Internationalization** — full **Slovene** (default) and **English** UI;
  auto-detects English from the `Accept-Language` header, otherwise Slovene.

**Navigation (placeholder screens)**

- Bottom tabs: **Fridge · Meal ideas · Shopping**. Meal ideas and Shopping are
  styled placeholders ("Coming soon") pending future phases.

**Planned next:** barcode scanning (camera →
[Open Food Facts](https://world.openfoodfacts.org/) lookup), AI meal ideas from
expiring items, an auto-populating shopping list, then optional reminders.

## Architecture

A layered Cargo workspace (ports & adapters) so the codebase stays maintainable
as it grows and multiple people work on it:

| Crate                   | Responsibility                                                                                                | Depends on        |
| ----------------------- | ------------------------------------------------------------------------------------------------------------ | ----------------- |
| `crates/fridgly-domain` | Business model & rules (`Item`, expiry/urgency, `Locale` + localised labels), the `ItemRepository` **port**. No framework deps. | —                 |
| `crates/fridgly-infra`  | Postgres **adapter** for the port, connection pool, migrations.                                              | `domain`, `sqlx`  |
| `crates/fridgly-web`    | Axum HTTP server, feature-sliced handlers, i18n catalog, Askama + htmx UI.                                   | `domain`, `infra` |

The web layer depends on the repository *trait*, never on Postgres directly, so
storage is swappable and the domain is unit-testable in isolation. Expiry labels
("in 5d" / "čez 5d") live in the domain because they are business logic;
UI-chrome translations and locale detection live in the web layer.

```
crates/fridgly-web/src/
├── main.rs               # composition root: config → pool → migrate → serve
├── config.rs             # env-driven configuration
├── state.rs              # shared AppState (repository behind a trait object)
├── error.rs              # AppError → HTTP status mapping
├── i18n.rs               # UI string catalog (EN/SL) + per-request locale detection
├── app.rs                # router assembly (+ dev live-reload)
└── features/
    ├── items/            # fridge feature: routes, forms, view models, handlers
    └── pages.rs          # Meal ideas / Shopping placeholders + /lang switcher

crates/fridgly-web/templates/   # Askama: base, index, list, macros, meals, shopping, …
crates/fridgly-web/static/      # style.css, app.js, htmx.min.js (vendored)
```

## Tech stack

Rust · Axum · Postgres (via `sqlx`) · Askama templates · htmx · Inter (web font).
Front-end interactions (add sheet, quantity stepper, expiry chips) are a small
`static/app.js`; everything else is server-rendered HTML swapped by htmx.

## Getting started

Prerequisites: the pinned Rust toolchain (installed automatically via
[`rust-toolchain.toml`](rust-toolchain.toml)) and Docker.

```bash
# 1. Start Postgres
docker compose up -d          # or: make db-up

# 2. Configure environment
cp .env.example .env

# 3. Run (migrations apply automatically on startup)
cargo run                     # or: make run

# App: http://localhost:3000
```

## Development

### Hot reload

`make dev` gives a nodemon-style loop: it rebuilds and restarts the server on any
change under `crates/` or `migrations/`, and — via the `dev` feature's
[`tower-livereload`](https://crates.io/crates/tower-livereload) layer — refreshes
the browser automatically once the server is back up.

```bash
cargo install cargo-watch     # one-time
make dev                      # watch + rebuild + live browser refresh
```

Notes:

- Askama templates and Rust are compiled into the binary, so editing them
  triggers a recompile (~2–3s incremental). Files in `static/` (CSS/JS) are
  served from disk and update without a rebuild.
- Live-reload is **only** compiled in under `--features dev`; it is never part of
  a release build. The reload script is skipped for htmx fragment responses so
  swaps aren't polluted.

### Quality gates

```bash
cargo test          # unit tests (domain rules, i18n labels, …)
cargo fmt           # format
cargo clippy        # lint
make check          # fmt-check + clippy + test (mirrors CI)
```

CI runs the same `fmt --check`, `clippy -D warnings`, and `test` on every push /
PR (see [`.github/workflows/ci.yml`](.github/workflows/ci.yml)).

## Internationalization

- Supported locales: `sl` (default), `en`.
- Resolution order per request: **`lang` cookie → `Accept-Language` header →
  Slovene**.
- The visible language switcher is currently hidden, but the machinery remains:
  `GET /lang/:code` persists a choice in a year-long cookie, and the
  `lang_switch` template macro can be dropped back into a topbar to re-enable it.
- Adding a language = one `const` in [`i18n.rs`](crates/fridgly-web/src/i18n.rs)
  plus one arm in the domain's `expiry_label`. No template changes.

## Configuration

Environment variables (see [`.env.example`](.env.example)):

| Variable             | Default                          | Purpose                                |
| -------------------- | -------------------------------- | -------------------------------------- |
| `DATABASE_URL`       | — (required)                     | Postgres connection string.            |
| `BIND_ADDR`          | `0.0.0.0:3000`                   | HTTP bind address.                     |
| `DB_MAX_CONNECTIONS` | `5`                              | Connection pool size.                  |
| `STATIC_DIR`         | `crates/fridgly-web/static`      | Directory served at `/static`.         |
| `RUST_LOG`           | `info,fridgly_web=debug`         | `tracing` filter.                      |

> Local Postgres runs on host port **5433** (via `docker compose`) to avoid
> clashing with a system Postgres on 5432.

## Deployment

Database migrations live in [`migrations/`](migrations/) and are embedded into
the binary at build time, so a release deployment needs only the binary + static
assets. A multi-stage [`Dockerfile`](Dockerfile) builds a slim runtime image:

```bash
docker build -t fridgly .
docker run -p 3000:3000 -e DATABASE_URL=... fridgly
```
