> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

## What and why

Fixes #5786 (follow-up of #5682). On main, a custom `RenderSetup` submitted through `SpecialModelSubmitOptions::with_render_type(SpecialModelRenderType::Custom(..))` reaches the item quads only for translucency, `LayeringTransform` and `TextureTransform::Offset`. The four settings the issue names are accepted and resolved but not drawn: the glint texture transforms, `use_lightmap`, `OutlineProperty` and a mod `pipeline`. Tinkers' Construct's glint render types need the first. This wires all four into the item draw, as vanilla 26.3's `RenderType.draw` and `OutlineBufferSource.getBuffer` apply them:

- `TextureTransform::Glint`, `EntityGlint`, `ArmorEntityGlint`: the quads' UVs scroll and scale with the frame's glint clock (`TextureTransform.createMatrix` / `setupGlintTexturing`, scales 8, 0.5 and 0.16, through the same `Glint::uv` the glint passes are checked against). `ItemRenderer` keeps the frame's `GlintFrame`, published once per frame where `ClientGame` captures it.
- `use_lightmap`: a render type without it has no `Sampler2` bound, so its quads draw at `LightCoordsUtil.FULL_BRIGHT` (`ResolvedRenderType::uses_lightmap`; of the built-ins, `eyes` and `lightning` have none).
- `OutlineProperty`: a glowing dropped item or item display outlines only its layers whose render type is not `NONE` (`ResolvedRenderType::affects_outline`, `ItemRenderState::outline_layers`). When every layer is outlined as drawn, the outline reuses the drawn vertices as before, with no second submit.
- `pipeline` naming a registered mod pipeline: a dropped item's or item display's quads of that render type are drawn by that `RenderPipeline` with the mods' custom geometry (`ItemQuad::mod_pipeline`, `PipelineRouting`, `ClientGame::submit_item_pipeline_quads`), when the pipeline takes the default `PositionTexColor` format and quads. Other sinks (GUI, hand, item frame) and other vertex formats keep the item shaders.
- A dropped item layer with a custom render type stays off the GPU skin path (`ItemRenderState::submit_skinned`), which applies none of these settings (nor main's `Offset`).

Draws with no custom render type do no new work: each per-layer check is an `Option` test on `layer.render_type`, the glint clock is read only for a glint transform, and the routing list is empty unless a quad names a pipeline.

Files:
- `crates/sunrise-client-render/src/mod_pipelines.rs`: `ResolvedRenderType::{uses_lightmap, affects_outline, mod_pipeline_slot}` and tests.
- `crates/sunrise-client/src/item_render.rs`: `transform_uv`, full-bright `packed_light`, `ItemQuad::mod_pipeline`, `ItemRenderState::outline_layers`, `ItemRenderer::set_glint_frame`, `PipelineItemQuad`, `PipelineRouting`, the skin-path refusal. `gui.rs`/`special.rs` literals gain `mod_pipeline: None`.
- `crates/sunrise-client-gameplay/src/game/interface/item_draw.rs`: `submit_item_world_routed`, `submit_item_world_joml_routed`, `routes_item_pipeline`, `submit_item_pipeline_quads`, and the dropped-item outline. `entities/text_displays.rs`: the item display's routed draw and outline. `client_side.rs`/`game_client.rs`: the `item_pipeline_quads` frame list and the glint-frame publish. `modding/native_custom_geometry.rs`: the routed quads draw with the custom geometry.
- `crates/sunrise-mod-vocab/src/client/render_pipeline.rs`: rustdoc on `RenderSetup`'s fields. `crates/sunrise-mod-vocab/changes/render-setup-item-draws.toml`: the change note.
- `docs/modding/src/how-to/client/special-item-renderers.md`: what a custom render type's setup does to the submitted quads.

**Behaviour change for mods**: `RenderSetup::new` already defaults to vanilla's builder values, `use_lightmap` false and `outline` `NONE`. Now that item draws honour them, a custom render type that set neither draws full bright and does not glow. The change note says to call `.use_lightmap(true)` and `.outline(OutlineProperty::AffectsOutline { texture: None })` to keep the old look.

## Kind

- [x] feature: something new for players (listed under New in the release notes)

## Layer

Client renderer (`sunrise-client-render`, `sunrise-client-lib`'s `item_render`, `sunrise-client-gameplay`). Mod API vocab: rustdoc and a change note only, no signature change.

## Vanilla reference

- `RenderType.draw` (binds `RenderSetup.pipeline`, `Sampler2` only with `useLightmap`, `TextureTransform.createMatrix` as the texture matrix), `RenderSetup.builder` defaults.
- `TextureTransform.GLINT_TEXTURING` / `ENTITY_GLINT_TEXTURING` / `ARMOR_ENTITY_GLINT_TEXTURING` (`setupGlintTexturing` with scales 8, 0.5, 0.16).
- `OutlineBufferSource.getBuffer`, `RenderSetup.OutlineProperty` (`NONE`, `IS_OUTLINE`, `AFFECTS_OUTLINE`), `RenderType.outline()`.
- `LightCoordsUtil.FULL_BRIGHT`.

## Self-check

Head `d8f74beb0bba5088df78e1e266193364fa89b3cb`, on Windows 11 (Rust 1.100; test resources prepared by main 4a6a5a9960d).

1. New and changed tests: `cargo test --locked --no-fail-fast -p sunrise-client-render -p sunrise-client-lib --lib -- --include-ignored mod_pipelines::tests::a_render_types_lightmap_and_outline_follow_its_setup mod_pipelines::tests::custom_render_types_validate_resolve_and_reject_stale_handles item_render::glint_tests::a_render_types_texture_transform_moves_the_quads_uvs item_render::glint_tests::a_custom_render_types_lightmap_pipeline_and_outline_reach_the_item_draw` (exit 0):

```
running 2 tests
test item_render::glint_tests::a_render_types_texture_transform_moves_the_quads_uvs ... ok
test item_render::glint_tests::a_custom_render_types_lightmap_pipeline_and_outline_reach_the_item_draw ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1022 filtered out; finished in 0.03s
running 2 tests
test mod_pipelines::tests::a_render_types_lightmap_and_outline_follow_its_setup ... ok
test mod_pipelines::tests::custom_render_types_validate_resolve_and_reject_stale_handles ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2614 filtered out; finished in 0.01s
```

2. Touched crates' tests: `cargo test --locked --no-fail-fast  -p sunrise-client-render -p sunrise-client-lib -p sunrise-client-gameplay -p sunrise-mod-vocab  --lib` (exit 101):

```
test result: ok. 804 passed; 0 failed; 279 ignored; 0 measured; 0 filtered out; finished in 3.93s
test result: ok. 944 passed; 0 failed; 80 ignored; 0 measured; 0 filtered out; finished in 192.65s
test glsl_spirv_dump_tests::glsl_spirv_dump_matches_the_committed_fixture ... FAILED
test light_storage_tests::light_reads_follow_vanilla_storage ... FAILED
thread 'glsl_spirv_dump_tests::glsl_spirv_dump_matches_the_committed_fixture' (32840) panicked at crates\sunrise-client-render\src\glsl_spirv_dump_tests.rs:132:26:
thread 'light_storage_tests::light_reads_follow_vanilla_storage' (10924) panicked at crates\sunrise-client-render\src\light_storage_tests.rs:82:5:
test result: FAILED. 2073 passed; 2 failed; 541 ignored; 0 measured; 0 filtered out; finished in 27.73s
test result: ok. 552 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 2.89s
```

3. `cargo clippy --locked -p sunrise-client-render -p sunrise-client-lib -p sunrise-client-gameplay -p sunrise-mod-vocab --all-targets -- -D warnings` with Rust 1.99.1, the CI toolchain (exit 101):

Outside this change: step 2's failures are `sunrise-client-render`'s `glsl_spirv_dump_matches_the_committed_fixture` and `light_reads_follow_vanilla_storage`, which fail the same way on main on this Windows host. Step 3 stops only on main's own test code in `sunrise-client-gameplay` (`native_block_sprites.rs`'s unused `Marker`/`Source` fixture), tracked in #7096; everything this PR changes passed clippy.

The full `tools/pr_ready.py --quick` block for this head follows in a comment.

- New tests:
  - `sunrise-client-render` `mod_pipelines::tests::a_render_types_lightmap_and_outline_follow_its_setup`: custom setups and the built-ins' lightmap and outline answers.
  - `sunrise-client-render` `mod_pipelines::tests::custom_render_types_validate_resolve_and_reject_stale_handles` (extended): the resolved type carries the setup's lightmap, outline and pipeline slot.
  - `sunrise-client-lib` `item_render::glint_tests::a_render_types_texture_transform_moves_the_quads_uvs`: `Default`, `Offset` and the three glint transforms at clock zero and after 1000 ms.
  - `sunrise-client-lib` `item_render::glint_tests::a_custom_render_types_lightmap_pipeline_and_outline_reach_the_item_draw`: full-bright vertices for an unlit layer, quads routed to the pipeline or drawn with the item shaders, and the outline layers.
- Python checks run here without a build: `tests.test_mod_api_boundary`, `tests.test_recursive_wire`, `tests.test_mod_api_struct_kinds` (35 tests, OK), `tools/hotspots/check_lists.py`, `tools/hotspots/check_stand_ins.py`, `tools/ci/check_ignored_test_rows.py`. rustfmt --edition 2024 was run on every changed .rs file.

Tried in game: no.
