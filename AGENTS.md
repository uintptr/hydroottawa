# AGENTS.md

Rust workspace: a CLI that fetches hourly electricity usage from Hydro
Ottawa's myAccount API and either prints tables or publishes to Home
Assistant via MQTT.

## Style guide

`CLAUDE_RUST.md` (repo root) applies to all Rust code here; follow it.
Ignore its PyO3/maturin, WASM, FFI, and web/data-framework sections —
none of those exist in this repo. Hard rules from it to keep in mind:

- No `#[allow(clippy::indexing_slicing)]` or `#[allow(unused_mut)]`.
- Public string/path params take generic `AsRef` bounds with `where`
  clauses (see `HoAuth::new`, `mqtt_publish`).
- Public items have doc comments; keep them in sync when editing.
- Run `cargo fmt` after edits.

## Layout

- `hydroottawa-api/` — library wrapping the API. Auth (`auth.rs`) is
  AWS Cognito SRP against a hardcoded pool/client ID, then a token
  exchange at `/app-token`; the JWT arrives in the
  `x-amzn-remapped-authorization` **response header**, not the body.
- `hydroottawa/` — the CLI. `lib.rs` exposes `display` (table output)
  and `mqtt_pub` (Home Assistant publishing); `main.rs` (clap
  entrypoint) consumes both from the lib.

## Conventions

- No async: HTTP is ureq (blocking) and MQTT uses rumqttc's blocking
  `Client`/`Connection`. There is no `#[tokio::main]` and no `.await`
  anywhere; tokio only survives as a transitive dep of rumqttc. Don't
  reintroduce an async runtime.
- Error handling: `hydroottawa-api` uses thiserror
  (`error::Error`/`Result`); anyhow is binary-only (`main.rs`,
  `mqtt_pub.rs`). Do not add anyhow to the library — it was removed
  deliberately (see git history).
- Workspace clippy lints: pedantic plus `unwrap_used`, `expect_used`,
  `arithmetic_side_effects`, `needless_pass_by_ref_mut`. The tree is
  currently clippy-clean; keep it that way.

## Verify

There are no tests and no CI (`cargo test` is a no-op). Check changes
with:

    cargo fmt --check
    cargo build --workspace
    cargo clippy --workspace --all-targets -- -D warnings

An end-to-end run requires live Hydro Ottawa credentials:

    HO_PASSWORD=... cargo run -- --username user@example.com

Without `HO_PASSWORD` the CLI prompts interactively (dialoguer) and
cannot run unattended. `--date` defaults to yesterday. `--mqtt <host>`
(not shown in the README) publishes Home Assistant discovery/state
topics instead of printing tables.

## Cross-compile

`.cargo/config.toml` sets the linker for `aarch64-unknown-linux-gnu` —
aarch64 is the intended deploy target (README plans a systemd service);
keep that setting. ureq is on rustls (its default), so no OpenSSL
cross-setup is needed — keep it that way.
