> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

## What and why

Since the gameplay split (#6527), `main` does not build on Windows (CI's "prebuilt SDK library has warnings or errors for: windows"). Two `sunrise-level` module gates are the wrong way round for what uses them:

- `pub mod perf_capture;` has no gate, but `perf_capture.rs` imports `sunrise_threads::heap_sample` and `sunrise_threads::profile`, which exist only on Linux. Its one user, `sunrise-sim`'s `server_perf::samples::linux`, is already Linux-only, and `sunrise-client-render` already gates its re-export. This gates the module and `sunrise-sim`'s re-export of it to `target_os = "linux"`.
- `pub mod api_context;` is gated to Linux (since 938fc1c750 moved it into `sunrise-level`), but it holds only plain load, loop and worldgen adapters and `sunrise-sim` uses it on every platform (`save/chunk_disk.rs`, `crafting/ingredient/matching.rs`, `crafting/mod_kinds.rs`). This removes that gate.

No behaviour change on Linux.

## Kind

- [x] fix: no label

## Layer

Build configuration of `sunrise-level` and `sunrise-sim` (module `cfg`s only).

## Vanilla reference

None: build fix.

## Self-check

On Windows 11 (Rust 1.100), at this head, on main ba8bf2d2f4: `cargo check --locked -p sunrise-client --lib --tests` passes (it fails on `main` without this change, first on `sunrise_threads::heap_sample` in `perf_capture.rs`, then on `sunrise_level::api_context` in `sunrise-sim`). rustfmt, `check_public_tree`, `check_repository_hygiene`, `check_layers`, `check_addresses`, `check_lists`, `check_stand_ins` and `check_launcher_mirrors` pass. The Linux build is unchanged: the only Linux-visible edit is removing a `cfg(target_os = "linux")` from a module that compiles everywhere.
