> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

On main, `cargo clippy -p sunrise-client-gameplay --all-targets -- -D warnings` fails on the crate's `lib test` target:

```
error: struct `Marker` is never constructed
   --> crates/sunrise-client-gameplay/src/game/modding/native_block_sprites.rs:224:12
error: struct `Source` is never constructed
   --> crates/sunrise-client-gameplay/src/game/modding/native_block_sprites.rs:229:12
```

They are the sprite-source fixture of that file's test module (`Marker`, and `Source` implementing `register_sprite_sources`). The tests that use them appear to be switched off since the gameplay split, so the fixture is unused. Either put the tests that build them back, or gate or remove the fixture, so clippy with `--all-targets -D warnings` passes on main again. Until then every PR that touches `sunrise-client-gameplay` reports this failure in its Self-check.
