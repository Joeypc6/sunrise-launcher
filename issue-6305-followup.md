> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

Title: Player touch API (#6305): pickup regressions with no mods installed, and five review points still on main

#6305 (`touch_item` / `touch_orb`, issue #6126) merged at 66724e0281, before its last review round. umceko's reviewer's seven points at that head are still in main's code:

**Regressions with no mods installed**
1. The host's orb pass changes ordinary pickup order: it applies Mending and XP before the orb's count is decremented (`crates/sunrise-gameplay/src/game/items/item_entities.rs`, the host pass near `:950` at that head). Vanilla's `ExperienceOrb.playerTouch` takes the orb's count first, then repairs and gives the rest.
2. The no-listener pickup path changed: it now does new resident lookups and removes an orb immediately (`item_entities.rs` near `:906`). With no mod listening to `pickup_xp`, the pass should be main's code from before #6305, unchanged.

**API correctness**
3. A parked guest can collect from another dimension (`native_item_entity_refs_server.rs:21`): a player who is not in the level being ticked should be refused.
4. Nested callbacks can duplicate items (`item_picked_up_server.rs:303`, `pickups_server.rs:194`): a `touch_item` of the item that is being picked up, made from that pickup's own listener, should take nothing.
5. `touch_item` misreports the accepted count on the guest branch (`:177` / `:203`): it should report what `Inventory.add` took, measured before any post listener runs.
6. Client notifications are missed (`:179`, `pickups_server.rs:210`): the API's take should send the take packet to every client tracking the entity.
7. The host and guest orb helpers duplicate the hook, residency, cooldown, Mending and XP steps (`item_entities.rs:916`); one shared path avoids the two drifting apart.

**A fix is ready.** Two signed-off commits on top of 66724e0281 fix all seven, and their tests pass on Windows (16 of 16 for the PR's tests): 1844b9a63f and d4bf4fdfbd. With no mod listening, the host orb pass is the pre-#6305 code word for word (the diff adds only a listener-only early branch and new functions). The other five points each have a test. They add `PickupError::OtherLevel`, `Err(Stale)` for an orb found only in a parked level, `take_item_tracked` (one line, in `sunrise-sim`) and a `touching_items` guard against nested duplicates. They can go up as a follow-up PR rebased on current main. Before that: one clippy warning (an unused import at `native_item_entity_refs_server.rs:247`).

**How to check:** with no mods installed, an orb touching a player with a damaged Mending item gives the same repair, XP and orb removal tick as before #6305. With a mod listening, the seven cases above each have a test in the fix.
