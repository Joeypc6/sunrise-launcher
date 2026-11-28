> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

## What and why

Fixes #5823. Ender IO's wired charger and enchant extractor show the player's armour and off hand beside the machine's slots, so worn gear can be charged or swapped (`WiredChargerMenu` calls `addArmorSlots`, which adds vanilla `ArmorSlot`s over `Inventory` 36 to 39). On main a block's mod menu type can only add the 36 inventory slots (`player_inventory`), so a port has to draw the gear itself and swap the cursor on the server, which loses vanilla's slot rules and the client's prediction.

This follows the shape agreed in the issue thread (A1 to A3):
- **A1.** A content menu type declares `"player_equipment": true` (default `false`; it needs `player_inventory`, and the 5 slots count toward the 32,768 total). That is authoritative on the server. The screen places the slots with new `MenuScreenLayout` fields `armor_x`/`armor_y` (top of the armour column, 18 apart) and `offhand_x`/`offhand_y`, in the same units as `inventory_x`/`inventory_y`.
- **A2.** Slot order: container slots, the 27 main and 9 hotbar slots, then the armour from head to feet over `Inventory` 39 to 36, then the off hand over 40, as `InventoryMenu` adds them. Existing menus' indices don't change, and a type without `player_equipment` is exactly as before: no new per-tick or per-click work.
- **A3.** The slots reuse the engine's existing `InventoryMenu` slot rules (`Role::Armor` and the off-hand slot in `sunrise_sim::menu`), on the server and in the client's prediction, because both build the same `Kind::Mod` slot list from the shared menu type registry:
  - an armour slot's `mayPlace` asks the stack's `equippable` component (`isEquippableInSlot` for the player) and the item's `can_equip` Parts, and holds one item;
  - `ArmorSlot.mayPickup`: armour with Curse of Binding (`prevent_armor_change`) stays on outside creative, for clicks, shift-clicks, number keys and drops;
  - `setByPlayer` runs `onEquipItem` (equip sound and event) through the existing `GuestMenuEffects.equipped` path;
  - shift-click from the inventory tries the container first, as today's mod-menu rule does. If the container takes nothing, `InventoryMenu.quickMoveStack`'s equipment rule applies: an equippable stack goes to its empty armour slot, else to an empty off hand. This is the one step beyond vanilla, and it is written in the doc comment. From an armour or off-hand slot, the stack goes to the inventory. A crafting result's shift-click still goes only to the inventory.
  - empty slots show vanilla's `container/slot/{helmet,chestplate,leggings,boots}` and `container/slot/shield` sprites, drawn by the existing inventory slot path.
- A blockless `open_mod_menu` of such a type is refused with the new `OpenMenuError::PlayerEquipmentWithoutBlock`, because the issue asks for a block's menu and the blockless click paths (display places, owned slots) don't model these slots.

Files:
- `crates/sunrise-mod-package/src/content.rs`: `MenuType.player_equipment`, its validation (`EquipmentWithoutInventory`, total slots).
- `crates/sunrise-level/src/containers.rs`: `ModMenuType.player_equipment`, `equipment_slots()`, `player_index()`. `slot_count()` counts the equipment slots. `register_mod_menu_types` takes `ModMenuType`s; the test-only tuple entry converts with `From` (no equipment), so the existing `with_mod_menu_types` call sites don't change.
- `crates/sunrise-gameplay-common/src/game/modding/mods/mod.rs`: registers the flag.
- `crates/sunrise-sim/src/menu.rs`: `Kind::Mod`'s slot list and `quickMoveStack`.
- `crates/sunrise-sim/src/lan/{menu,containers,guest}.rs`: the guest menu's slot count, the opening snapshot, `view_slot`, bundle selection and the click settle map these slots to the guest's armour and off hand, and the snapshot tells the inventory menu about them too (`equipment_told`, as `transferState` would).
- `crates/sunrise-gameplay/src/game/modding/native_menu_refs_server.rs`: `MenuRef` slot backings `Player(39..36)` and `Player(40)`. `native_menus.rs`: the blockless refusal. `network/lan_crafting.rs`: a machine click doesn't send a separate off-hand repair when the menu shows the off hand.
- Client (`sunrise-client-gameplay`, `sunrise-gameplay-common`): the remote slot map, the prediction's slot map (`MenuSlotMap.equipment`), `MenuRef::slot_kinds` (`Player(39..36)`, `Player(40)`), and the mod screen's declared positions, slot moves (`screen_slot`/`set_screen_slot` reach these slots too), hit-testing and drawing. The open check also refuses non-finite new coordinates.
- `crates/sunrise-mod-vocab`: the four layout fields, the new refusal, rustdoc for `set_screen_slot`/`ScreenSlotRefusal`, and the change note `changes/menu-player-equipment.toml`.
- Docs: `docs/modding/src/how-to/content-mods.md` (menu types) and `how-to/client/screens.md` (slot positions).
- Every `ModMenuType`/`MenuType` literal in tests gains `player_equipment: false`.

Not covered: `native_menu_quick_move` (`MenuQuickMove` Parts) is not asked about a shift-click on an armour or off-hand slot; the game's own routing answers it. That hook's `MenuSlot` has no place for equipment.

## Kind

- [x] feature: something new for players (listed under New in the release notes)

## Layer

Mod API (`sunrise-mod-vocab`, the content `MenuType` in `sunrise-mod-package`), with the shared menu engine in `sunrise-sim`/`sunrise-level`, the server in `sunrise-gameplay` and the client in `sunrise-client-gameplay`/`sunrise-gameplay-common`. No new host method, route or dependency.

## Vanilla reference

- `InventoryMenu` constructor: `ArmorSlot`s for `SLOT_IDS` head to feet over `Inventory` 39 to 36, then the off-hand slot over 40 (`setByPlayer` calls `onEquipItem(OFFHAND, ...)`, empty icon `EMPTY_ARMOR_SLOT_SHIELD`), and `InventoryMenu.quickMoveStack`'s equipment branch (`player.getEquipmentSlotForItem`, the empty armour slot `8 - getIndex()`, then slot 45).
- `ArmorSlot.mayPlace` (`owner.isEquippableInSlot`), `getMaxStackSize` 1, `mayPickup` (`EnchantmentHelper.has(stack, PREVENT_ARMOR_CHANGE)` unless creative), `setByPlayer` (`onEquipItem`), `getNoItemIcon`.
- `AbstractContainerMenu.moveItemStackTo`; `ChestMenu.quickMoveStack` (the existing mod-menu rule).
- Ender IO 26.3 `WiredChargerMenu.addArmorSlots` (the use case).

## Self-check

Head `4623e5f0207067528483e0b128e812add19f839f`, on Windows 11 (Rust 1.100; test resources prepared by main 4a6a5a9960d).

1. New and changed tests: `cargo test --locked --no-fail-fast -p sunrise-mod-package -p sunrise-sim -p sunrise-gameplay-common -p sunrise-client-gameplay --lib -- --include-ignored content::tests::a_menu_type_declares_the_players_equipment content::tests::menu_types_take_slots_data_slots_and_roles menu::tests::a_mod_menus_equipment_slots_are_the_players_armour_and_off_hand menu::tests::a_shift_click_in_a_mod_menu_fills_the_container_then_the_equipment menu::tests::a_mod_menus_armour_slot_keeps_cursed_armour_on_outside_creative lan::containers::tests::a_mod_menus_equipment_slots_show_the_guests_armour_and_off_hand game::remote::inventory::tests::a_mod_menus_equipment_slots_map_to_the_armour_and_off_hand game::client_side::native_menu_slot_kinds::tests::equipment_slots_are_the_players_armour_and_off_hand` (exit 0):

```
running 1 test
test game::client_side::native_menu_slot_kinds::tests::equipment_slots_are_the_players_armour_and_off_hand ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1083 filtered out; finished in 0.00s
running 1 test
test game::remote::inventory::tests::a_mod_menus_equipment_slots_map_to_the_armour_and_off_hand ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 513 filtered out; finished in 0.00s
running 2 tests
test content::tests::a_menu_type_declares_the_players_equipment ... ok
test content::tests::menu_types_take_slots_data_slots_and_roles ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 305 filtered out; finished in 0.12s
running 4 tests
test lan::containers::tests::a_mod_menus_equipment_slots_show_the_guests_armour_and_off_hand ... ok
test menu::tests::a_mod_menus_armour_slot_keeps_cursed_armour_on_outside_creative ... ok
test menu::tests::a_mod_menus_equipment_slots_are_the_players_armour_and_off_hand ... ok
test menu::tests::a_shift_click_in_a_mod_menu_fills_the_container_then_the_equipment ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 3713 filtered out; finished in 0.13s
```

2. Touched crates' tests: `cargo test --locked --no-fail-fast  -p sunrise-mod-package -p sunrise-level -p sunrise-sim -p sunrise-mod-vocab -p sunrise-gameplay-common -p sunrise-gameplay -p sunrise-client-gameplay -p sunrise-client  --lib` (exit 101):

```
test result: ok. 338 passed; 0 failed; 4490 ignored; 0 measured; 0 filtered out; finished in 25.47s
test result: ok. 832 passed; 0 failed; 279 ignored; 0 measured; 0 filtered out; finished in 5.23s
test game::mods::tests::the_test_process_does_not_run_in_the_crates_folder ... FAILED
thread 'game::mods::tests::the_test_process_does_not_run_in_the_crates_folder' (30448) panicked at crates\sunrise-gameplay\src\game\modding\mods\mod.rs:1988:9:
test result: FAILED. 607 passed; 1 failed; 438 ignored; 0 measured; 0 filtered out; finished in 3.48s
test result: ok. 504 passed; 0 failed; 10 ignored; 0 measured; 0 filtered out; finished in 21.88s
test result: ok. 437 passed; 0 failed; 25 ignored; 0 measured; 0 filtered out; finished in 2.48s
test holder::tests::windows_controller_quietly_declines_managed_mode_without_spawning ... FAILED
thread 'holder::tests::windows_controller_quietly_declines_managed_mode_without_spawning' (30552) panicked at crates\sunrise-mod-package\src\holder\tests.rs:71:18:
test result: FAILED. 274 passed; 1 failed; 32 ignored; 0 measured; 0 filtered out; finished in 8.98s
test result: ok. 552 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 1.67s
test result: ok. 2950 passed; 0 failed; 767 ignored; 0 measured; 0 filtered out; finished in 176.85s
```

3. `cargo clippy --locked -p sunrise-mod-package -p sunrise-level -p sunrise-sim -p sunrise-mod-vocab -p sunrise-gameplay-common -p sunrise-gameplay -p sunrise-client-gameplay -p sunrise-client --all-targets -- -D warnings` with Rust 1.99.1, the CI toolchain (exit 101):

Outside this change: step 2's two failures are `sunrise-gameplay`'s `game::mods::tests::the_test_process_does_not_run_in_the_crates_folder` (this host's cargo puts test binaries under `target/debug/build/`) and `holder::tests::windows_controller_quietly_declines_managed_mode_without_spawning` (in `sunrise-mod-package`'s holder module, which this PR doesn't change: a Windows-only holder test); both fail the same way on main on this host, and this PR doesn't change them. Step 3 stops only on main's own test code in `sunrise-client-gameplay` (`native_block_sprites.rs`'s unused `Marker`/`Source` fixture), tracked in #7096; everything this PR changes passed clippy.

Short check as accepted by the reviewers (changed tests + clippy on the touched crates); the full `tools/pr_ready.py --quick` block for this head follows in a comment.

- New tests (all unit tests, no prepared data):
  - `sunrise-mod-package` `content::tests::a_menu_type_declares_the_players_equipment`, and two refusal cases added to `content::tests::menu_types_take_slots_data_slots_and_roles` (equipment without inventory, total over 32,768).
  - `sunrise-sim` `menu::tests::a_mod_menus_equipment_slots_are_the_players_armour_and_off_hand`: 38 slots without the flag, 43 with it; slots 38 to 41 hold one of what equips there and nothing else, 42 (off hand) holds a full stack, and they write `Inventory` 39 and 40.
  - `sunrise-sim` `menu::tests::a_shift_click_in_a_mod_menu_fills_the_container_then_the_equipment` (A3's first two cases): the container first, then the empty head slot (heard as `head` equipped), not a worn slot, the off hand for an off-hand item, nothing for a plain stack. From the head slot the stack goes back to the inventory. A type without the flag puts nothing on.
  - `sunrise-sim` `menu::tests::a_mod_menus_armour_slot_keeps_cursed_armour_on_outside_creative` (A3's third case): Curse of Binding keeps the chest piece on for a click and a shift-click in survival; in creative it comes off.
  - `sunrise-sim` `lan::containers::tests::a_mod_menus_equipment_slots_show_the_guests_armour_and_off_hand`: the server's opening snapshot and `view_slot` show the guest's armour head to feet and off hand at 38 to 42, and the inventory menu's 5 and 45 are marked as told.
  - `sunrise-gameplay-common` `game::remote::inventory::tests::a_mod_menus_equipment_slots_map_to_the_armour_and_off_hand`: the client's slot map sends them to the screen's 500 to 504.
  - `sunrise-client-gameplay` `game::client_side::native_menu_slot_kinds::tests::equipment_slots_are_the_players_armour_and_off_hand`: `slot_kinds` reports `Player(39)` to `Player(36)`, then `Player(40)`.
- Python checks run here without a build: `tests.test_mod_api_boundary`, `tests.test_recursive_wire`, `tests.test_mod_api_struct_kinds` (35 tests, OK), `tools/hotspots/check_lists.py`, `tools/hotspots/check_stand_ins.py`, `tools/ci/check_ignored_test_rows.py`. rustfmt --edition 2024 was run on every changed .rs file.

Tried in game: no.
