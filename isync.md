> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

## The gap

`ClientGame::client_world_tables` returns `None` unconditionally (`crates/sunrise-client-gameplay/src/game/world/world_entry_client.rs:29`). The client's state therefore never holds `WorldTables`, and every client read of a data-pack table falls back to the game's built-in one. Two mod API reads show it (#6312):

- `BlockStates::is_suffocating` and `LevelRef::state_is_suffocating` read the client's suffocation rules (`GameItems::level_suffocation`), which are the built-in `#minecraft:causes_suffocation` binding.
- `BlockStates::is_correct_tool_for_drops` reads the client's item table (`GameItems::items`), whose block tags (`#minecraft:mineable/pickaxe` and the rest) are the built-in ones.

On the server both follow the world's data packs, through the host and inside a level callback. On the client a world pack that rebinds those tags changes nothing.

## What vanilla does

The server sends its tags to the client (`ClientboundUpdateTagsPacket`, on join and on `/reload`), and the client binds its registries' tags to them (`TagNetworkSerialization` / `RegistryAccess` tag update). Predicates the client evaluates, `BlockState.isSuffocating`'s tag test and `Tool.isCorrectForDrops`' `HolderSet<Block>` among them, then see the server's tags.

## Proposed fix

When the synced tags arrive (join and reload), build the client's world tables from them: rebind the suffocation rules to the synced `#minecraft:causes_suffocation` (as the server's `stage_world_tables` binds them from the world's pack) and give the client's item table the synced block tags. Return those tables from `client_world_tables`. The generated `ClientGame` forwarders (`SideBlockTables`) and the client's level views read the side's tables already, so they follow with no further change.

## How to test

The daily `cut_layout::game::server_side::native_entity_refs::tests::a_world_pack_decides_the_hosts_table_reads_on_its_side` (from #6312) stages a world pack that replaces `#minecraft:causes_suffocation` and `#minecraft:mineable/pickaxe` with dirt alone. Its client half asserts the client host keeps the built-in answers (stone suffocates, an iron pickaxe is correct for stone). With the fix, sync the server's tags to the client in that test; the client half then flips to the pack's answers (stone `Some(false)` for both, dirt `Some(true)`), matching the server's.
