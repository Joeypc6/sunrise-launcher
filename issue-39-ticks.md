> 🤖 Filed by Joeypc6's AI agent (Claude Code), not by Joeypc6.

## What happens

In a single-player world, no native mod block entity ticks on the logical server. This holds even when the type's `BlockEntityPart` implements `block_entity_tick` and the type has a ticker. The ticker selector (`ticker_when`) is never called, and `block_entity_tick` never runs. On a dedicated server the same mod ticks. Dev ba1f24e (before the split) ticked it in single player too.

## Cause, on main (70d397565a)

1. **Startup binding works.** `register_block_entity_types` (`crates/sunrise-gameplay-common/src/game/modding/mods/mod.rs`) binds a server ticker for every mod type (159 for one port), and `block_entity_types::install` returns `Ok`.
2. **The bake keeps only one side.** `prepare_mod_tickers` (`crates/sunrise-gameplay-common/src/game/modding/api_values/block_state.rs`) bakes the per-state ticker tables. It now takes an `ExecutionRole`:
   ```rust
   let eval_server = role.is_none_or(|r| r.is_logical_server());
   let eval_client = role.is_none_or(|r| r.is_client());
   ```
3. **In single player that side is the client.** The bake runs once, on the client's main thread, with `role = Some(Client)` (`resource_load.rs`: `resolve_block_definitions_for_role(assets.registry, role)`). So `eval_server` is false and the server table is empty. A diagnostic line from the engine:
   `prepare_mod_tickers thread "main": role=Some(Client) eval_server=false eval_client=true blocks_with_tickers=996`
4. **The integrated server reuses the client's bake.** It never prepares its own. `native_worldgen::retain_or_prepare` shares the first index across every registry interned from the same payload and metadata snapshot (`BlockRegistry::share_worker_resource`, keyed by the metadata pointer). So the integrated server ticks with the client's index, and that index's server table is empty.

A bake limited to one role can't be shared between roles. Before the split, `prepare_mod_tickers` took no role and baked both sides.

## Repro

1. Use any native mod whose `BlockEntityPart` implements `block_entity_tick`.
2. Place its block in a new single-player world.
3. `block_entity_tick` never runs. On a dedicated server, the same block ticks.

## Fix options

- **Minimal:** bake the server side for every role, and the client side for every role except a dedicated server. A local patch that does this restores ticking.
- **Cleaner:** key the shared worker resource by role, or by the sides this process hosts, so the integrated server bakes its own server table.
- **Or:** bake both sides whenever the index can be shared in-process, and limit the bake only on a dedicated server or a client that only joins remote servers.

## Tests

The block entity tick tests (`cut_layout/game/blocks/block_entity_ticks.rs`) prepare with no role, so they don't catch this. A test that prepares with `Some(Client)` and then ticks on the integrated server would.

## Related

- #6527 (the gameplay split), #2210 (its tracker).
- #6842: daily tests compiled out since the split.
