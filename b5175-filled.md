> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

## What and why

Fixes #5175. When Tinkers' Construct casts in a casting table or basin, the result fades in as it cools and glows orange while hot. `CastingBlockEntityRenderer` draws it through `CastingItemRenderTypeBuffer`, which swaps the item's solid and cut-out render types for a translucent entity type and wraps each `VertexConsumer` in Mantle's `TintedVertexBuilder` to multiply every vertex colour. On main, `BlockEntityDrawContext::item` takes no colour or alpha. `model(model, tint)` only tints faces that have a tint index, and item tint sources carry no alpha and work per stack. So the port draws the result opaque and only from halfway through the cooling.

Fix: a new `BlockEntityDrawContext::item_tinted(stack, display, owner_yaw, seed, argb)`. It is `item` with every vertex it draws multiplied by a straight-alpha `0xAARRGGBB`, tinted layers included. It follows the same shape as `GuiCanvas::item_tinted` (#5594).
- Below an alpha of 255, the item's plain (overlay 0) and foil (7) quads move to the translucent item sheet's overlays (`TRANSLUCENT_ITEM_OVERLAY`, `TRANSLUCENT_GLINT_ITEM_OVERLAY`), so they draw blended and sorted far to near. They lose the `OIT_ENTITY` tag, as `WorldSink` never sets it on that sheet. This stands for `CastingItemRenderTypeBuffer.getBuffer`'s swap. Quads already in the translucent sheet, and special overlays such as banner patterns, keep their material and only get the colour.
- Glint-only quads keep their colour: vanilla's glint render types ignore the vertex colour, and `entity.wgsl`'s `glint_over` already draws the per-vertex glint at its own strength.
- `0xFFFFFFFF` draws exactly what `item` draws, with no pass over the vertices. A zero alpha draws nothing and returns false, as `GuiCanvas::item_tinted` does.
- Display transform, owner, seed, `unlit` and light are `item`'s, because the call goes through `native_draw_item` unchanged.

`item` and every block entity renderer that doesn't call the new method have no new work: the colour pass runs only in `item_tinted`, only over the vertices that call appended, and only when the tint isn't opaque white.

Files:
- `crates/sunrise-mod-vocab/src/host/draw.rs`: the trait method and its rustdoc.
- `crates/sunrise-client-gameplay/src/game/host/draw_client.rs`: the host impl (stack and display lowering as `item` does them).
- `crates/sunrise-client-gameplay/src/game/modding/native_block_entity_draw.rs`: `ClientGame::native_draw_item_tinted` (`#[inline(always)]`), `tint_item_vertices` and the tests.
- `crates/sunrise-mod-vocab/changes/block-entity-item-tint.toml`: the change note.
- `docs/modding/src/how-to/client/model-drawing.md`: a paragraph and Tinkers' cooling colours as an example.
- `tools/ci/ignored-tests/daily-sunrise-client-block-entity-item-tint.txt`: the daily row.

## Kind

- [x] feature: something new for players (listed under New in the release notes)

## Layer

Mod API (`sunrise-mod-vocab`: one new `BlockEntityDrawContext` method with a default), with the client implementation in `sunrise-client-gameplay`. No new route, pipeline, material or dependency. The existing translucent item sheet is reused.

## Vanilla reference

- `MultiBufferSource.getBuffer(RenderType)` and `VertexConsumer.setColor` as a mod's buffer source wraps them. Tinkers' Construct 1.20.1 `smeltery.client.render.CastingBlockEntityRenderer` and `smeltery.client.util.CastingItemRenderTypeBuffer`, Mantle `client.render.TintedVertexBuilder`.
- `Sheets.translucentItemSheet()` / `RenderTypes.entityTranslucent` (the translucent material the faded quads move to), `ItemStackRenderState.submit`.
- `RenderPipelines.GLINT` (`glint.fsh` ignores the vertex colour).

## Self-check

Head `7d367399bc73e3560572a4195bb67e58914e9753`, on Windows 11 (Rust 1.100; test resources prepared by main 4a6a5a9960d).

1. New and changed tests: `cargo test --locked --no-fail-fast -p sunrise-client-gameplay --lib -- --include-ignored game::client_side::native_block_entity_draw::tests::a_faded_item_moves_to_the_translucent_item_sheet_with_its_colours_multiplied game::client_side::native_block_entity_draw::tests::a_tinted_item_multiplies_its_colours_and_fades_in_the_translucent_item_sheet` (exit 0):

```
running 2 tests
test game::client_side::native_block_entity_draw::tests::a_faded_item_moves_to_the_translucent_item_sheet_with_its_colours_multiplied ... ok
test game::client_side::native_block_entity_draw::tests::a_tinted_item_multiplies_its_colours_and_fades_in_the_translucent_item_sheet ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1083 filtered out; finished in 22.62s
```

2. Touched crates' tests: `cargo test --locked --no-fail-fast  -p sunrise-mod-vocab -p sunrise-client-gameplay  --lib` (exit 0):

```
test result: ok. 805 passed; 0 failed; 280 ignored; 0 measured; 0 filtered out; finished in 4.25s
test result: ok. 552 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.87s
```

3. `cargo clippy --locked -p sunrise-mod-vocab -p sunrise-client-gameplay --all-targets -- -D warnings` with Rust 1.99.1, the CI toolchain (exit 101):

Step 3 stops only on main's own test code in `sunrise-client-gameplay` (`native_block_sprites.rs`'s unused `Marker`/`Source` fixture), tracked in #7096; everything this PR changes passed clippy.

The full `tools/pr_ready.py --quick` block for this head follows in a comment.

- New tests:
  - `sunrise-client-gameplay` `game::client_side::native_block_entity_draw::tests::a_faded_item_moves_to_the_translucent_item_sheet_with_its_colours_multiplied` (unit test): plain and foil quads move to the translucent sheet and keep their HDR tier, with `OIT_ENTITY` cleared. Colours are multiplied in vertex (ABGR) order. Glint-only quads keep theirs. An opaque tint changes only the colours.
  - `sunrise-client-gameplay` `game::client_side::native_block_entity_draw::tests::a_tinted_item_multiplies_its_colours_and_fades_in_the_translucent_item_sheet` (daily, prepared resources via `with_client_game`): a faded unlit stick draws white times the tint in the translucent item sheet, opaque white draws exactly what `native_draw_item` draws, and a zero alpha draws nothing.
- Python checks run here without a build: `tests.test_mod_api_boundary`, `tests.test_recursive_wire`, `tests.test_mod_api_struct_kinds` (35 tests, OK), `tools/hotspots/check_lists.py`, `tools/hotspots/check_stand_ins.py`, `tools/ci/check_ignored_test_rows.py`. rustfmt --edition 2024 was run on every changed .rs file.

Tried in game: no.
