> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

## What and why

Part of #5433 (client fluid type extensions). On main a mod's world fluid already has its sprites and tint (`data/<id>/sunrise/fluid/`'s `sprite`/`tint`, `register_fluid_models`, NeoForge's `getStillTexture`/`getFlowingTexture`/`getOverlayTexture`/`getTintColor`). Three things are missing. A camera inside the fluid sees no overlay. It gets no fog of the fluid's own: a mod fluid is neither water nor lava, so `Camera.getFluidInCamera` answers `ATMOSPHERIC`, as vanilla and NeoForge do. And only the global `compute_fog_color`/`render_fog` hooks can change the fog, without knowing the fluid. Tinkers' Construct needs all three: each Mantle `fluid_texture` file names a `camera` texture and `fog` start/end and colour for its molten metals, slimes and soups.

This adds them as data, on the same fluid file that holds the rest of the look. It adds a `camera` object with every field optional:

| Field | NeoForge `IClientFluidTypeExtensions` | Applied where |
|---|---|---|
| `overlay`: texture id `ns:path` (`assets/ns/textures/path.png`) | `getRenderOverlayTexture` + the default `renderOverlay`/`extractOverlay` (`ScreenEffectRenderer.renderFluid`) | Water's overlay quad, the same brightness, alpha 0.1, yaw/pitch scroll and `.mcmeta` sampling, with the fluid's texture. It is drawn while the first-person eye (`getEyeY() - 0.11111111`, as `updateFluidOnEyes` reads it) is below the fluid's surface, never for a spectator. |
| `fog_color`: `0xRRGGBB` | `modifyFogColor` | Fog colour while the camera is below the surface. The alpha stays. It applies before `compute_fog_color`, as `ClientHooks.getFogColor` runs before `ViewportEvent.ComputeFogColor`. |
| `fog_start`, `fog_end`: blocks, set together, end > start | `modifyFogRender` | The fog's environmental start and end while the camera is below the surface. It applies before `render_fog` and after Sunrise's Fog option, which clears only the open air's fog. |

"Below the surface" is NeoForge's test: the fluid in the camera's block, with the camera under `FluidState.getHeight` there (1 under the same fluid, else amount/9).

Water and lava never reach this code. A fluid that sets no `camera` costs a frame one relaxed atomic load (`fluid_models::any_camera_look`). Only when some look sets one does a frame read the camera block's facts and do one hash lookup. No per-tick work is added.

Files:
- `crates/sunrise-mod-package/src/content.rs`: `FluidCamera` (fields, `check`, `is_set`, `fog`), `FluidLook::camera`, the `camera` key in the fluid file, and a parse/refusal test. The goldens and the `fluid_full` corpus file are updated.
- `crates/sunrise-client-render/src/fluid_models.rs`: camera looks kept by liquid block id, `any_camera_look`, `with_camera_look`, `camera_overlay_textures`.
- `crates/sunrise-client-render/src/fog.rs`: `mod_fluid_at` (NeoForge's camera-in-fluid test).
- `crates/sunrise-client-render/src/screen_effects.rs`: `ScreenInputs::overlay_texture`, so the submerged overlay draws a mod fluid's texture.
- `crates/sunrise-client-render/src/hud/atlas_pack.rs`: the GUI atlas packs the named overlay textures under their ids, wherever they are in the namespace.
- `crates/sunrise-client-gameplay/src/game/interface/render_tick.rs`: `fluid_camera_fog` in `frame_fog` and `eye_fluid_overlay` in `screen_quads` (`#[inline(always)]`).
- `docs/modding/src/how-to/content-mods.md`, `docs/api/data-formats.md`: the `camera` object.
- The `FluidLook` literals in other crates gain `camera: Default::default()`.

There is no `crates/sunrise-mod-vocab/changes/` note: the mod API surface (`sunrise-mod-vocab`) is unchanged. The fluid file is content data in `sunrise-mod-package`.

Not in this PR, and why. The issue also asks for a callback per method, with a panic falling back for the frame. A mod that draws its whole submerged overlay itself (`extractOverlay`) needs a new client Part hook with a camera draw context. Fog that changes from frame to frame (`modifyFogColor`/`modifyFogRender` reading the camera or time) also needs one. The data covers the default `extractOverlay` and every static answer, which is what Tinkers' Mantle files state. Dynamic fog is reachable today through the global `compute_fog_color`/`render_fog` hooks, since a mod can read the fluid at the camera there. A fluid Part hook for those callbacks is a follow-up if a port needs it.

## Kind

- [x] feature: something new for players (listed under New in the release notes)

## Layer

Content data (`sunrise-mod-package`: a new optional key in the fluid file) and the client renderer (`sunrise-client-render`, `sunrise-client-gameplay`). No mod API vocab, hook, route, pipeline or dependency change.

## Vanilla reference

- NeoForge 26.3 `IClientFluidTypeExtensions.getRenderOverlayTexture`, `extractOverlay` (default: `ScreenEffectRenderer.renderFluid` with that texture), `modifyFogColor`, `modifyFogRender`. `ClientHooks.getFogColor` and `onFogRender` ask them for the fluid at `camera.getBlockPosition()` when the camera is below `FluidState.getHeight`, before `ViewportEvent.ComputeFogColor`/`RenderFog`.
- `Entity.updateFluidOnEyes` (`getEyeY() - 0.11111111F`, NeoForge's `forgeFluidTypeOnEyes`) and `ScreenEffectRenderer.renderWater`/`renderFluid` (brightness at the eye, alpha 0.1, the yaw/pitch scroll).
- `FlowingFluid.getHeight` (1 with the same fluid above, else `getOwnHeight`, amount/9).
- Tinkers' Construct / Mantle `fluid_texture` JSON (`camera`, `fog.start`, `fog.end`), which a port maps onto `camera.overlay`, `camera.fog_start` and `camera.fog_end`, with `camera.fog_color` computed at conversion where Mantle's `calculate_color` is set.

## Self-check

Head `54dbaff23e05c0c6c0d2f306bfac94c74fadf090`, on Windows 11 (Rust 1.100; test resources prepared by main 4a6a5a9960d).

1. New and changed tests: `cargo test --locked --no-fail-fast -p sunrise-mod-package -p sunrise-client-render --lib -- --include-ignored content::tests::fluids_ fluid_models::tests:: fog::tests::the_camera_is_in_a_mod_fluid_below_its_surface screen_effects::tests:: && cargo test --locked --no-fail-fast -p sunrise-mod-package --test content_golden` (exit 0):

```
running 22 tests
test fluid_models::tests::a_model_names_what_the_look_cannot_and_keeps_the_looks_tint ... ok
test fluid_models::tests::a_model_for_no_world_fluid_draws_nothing_and_a_bad_sprite_is_ignored ... ok
test fluid_models::tests::a_sprite_without_a_tint_is_drawn_as_it_is ... ok
test fluid_models::tests::a_look_alone_draws_its_sprite_and_tint_on_every_surface ... ok
test fluid_models::tests::no_sprite_draws_waters_textures_with_the_look_tint_or_the_biomes ... ok
test fluid_models::tests::vanilla_buckets_hold_their_fluid ... ok
test fluid_models::tests::vanilla_fluids_draw_their_still_sprite_with_their_item_tint ... ok
test fog::tests::the_camera_is_in_a_mod_fluid_below_its_surface ... ok
test screen_effects::tests::camera_quads_refuse_what_is_not_finite ... ok
test fluid_models::tests::a_mod_fluids_flowing_form_draws_its_source_appearance ... ok
test fluid_models::tests::a_mod_fluid_draws_its_world_still_sprite_and_tint ... ok
test fluid_models::tests::camera_looks_are_kept_by_liquid_block ... ok
test fluid_models::tests::a_reload_changes_what_an_items_fluid_draws ... ok
test screen_effects::tests::lightmap_brightness_curve ... ok
test fluid_models::tests::a_tank_only_fluid_draws_its_look_in_an_item_but_not_in_the_world ... ok
test screen_effects::tests::camera_quads_keep_their_depth_for_perspective ... ok
test screen_effects::tests::a_mod_fluid_overlay_draws_its_texture_as_water_does ... ok
test screen_effects::tests::custom_quads_go_before_their_overlays ... ok
test screen_effects::tests::overlays_need_their_sprites ... ok
test screen_effects::tests::mcmeta_texture_section ... ok
test screen_effects::tests::portal_intensity_ramps ... ok
test screen_effects::tests::standalone_textures_sample_by_their_metadata ... ok
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 2593 filtered out; finished in 0.01s
running 4 tests
test content::tests::fluids_declare_their_temperature_density_and_viscosity ... ok
test content::tests::fluids_declare_a_sprite_and_a_tint ... ok
test content::tests::fluids_declare_what_a_camera_inside_them_sees ... ok
test content::tests::fluids_name_their_bucket_flowing_form_and_light ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 225 filtered out; finished in 0.03s
running 3 tests
test daily_content_golden ... ignored, daily
test content_golden ... ok
test content_golden_is_deterministic ... ok
test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.46s
```

2. Touched crates' tests: `cargo test --locked --no-fail-fast  -p sunrise-mod-package -p sunrise-client-render -p sunrise-client-gameplay -p sunrise-gameplay-common -p sunrise-client  --lib` (exit 101):

```
test result: ok. 338 passed; 0 failed; 4490 ignored; 0 measured; 0 filtered out; finished in 21.67s
test result: ok. 831 passed; 0 failed; 279 ignored; 0 measured; 0 filtered out; finished in 4.00s
test glsl_spirv_dump_tests::glsl_spirv_dump_matches_the_committed_fixture ... FAILED
test light_storage_tests::light_reads_follow_vanilla_storage ... FAILED
thread 'glsl_spirv_dump_tests::glsl_spirv_dump_matches_the_committed_fixture' (2508) panicked at crates\sunrise-client-render\src\glsl_spirv_dump_tests.rs:132:26:
thread 'light_storage_tests::light_reads_follow_vanilla_storage' (20868) panicked at crates\sunrise-client-render\src\light_storage_tests.rs:82:5:
test result: FAILED. 2075 passed; 2 failed; 541 ignored; 0 measured; 0 filtered out; finished in 16.72s
test result: ok. 503 passed; 0 failed; 10 ignored; 0 measured; 0 filtered out; finished in 15.15s
test holder::tests::windows_controller_quietly_declines_managed_mode_without_spawning ... FAILED
thread 'holder::tests::windows_controller_quietly_declines_managed_mode_without_spawning' (33120) panicked at crates\sunrise-mod-package\src\holder\tests.rs:71:18:
test result: FAILED. 274 passed; 1 failed; 32 ignored; 0 measured; 0 filtered out; finished in 2.83s
```

3. `cargo clippy --locked -p sunrise-mod-package -p sunrise-client-render -p sunrise-client-gameplay -p sunrise-gameplay-common -p sunrise-client --all-targets -- -D warnings` with Rust 1.99.1, the CI toolchain (exit 101):

```
error: struct `Marker` is never constructed
   --> crates/sunrise-client-gameplay/src/game/modding/native_block_sprites.rs:224:12
error: struct `Source` is never constructed
   --> crates/sunrise-client-gameplay/src/game/modding/native_block_sprites.rs:229:12
error: could not compile `sunrise-client-gameplay` (lib test) due to 2 previous errors
```

Outside this change: step 2's failures are `sunrise-client-render`'s `glsl_spirv_dump_matches_the_committed_fixture` and `light_reads_follow_vanilla_storage` and `sunrise-client`'s `holder::tests::windows_controller_quietly_declines_managed_mode_without_spawning`, which fail the same way on main on this Windows host; this PR changes none of them. Step 3 stops only on main's own test code above (the `lib test` target of a file this PR doesn't change; `-D warnings` fails on it at main too). Everything this PR changes passed clippy. CI's Linux runs are the check for the rest.

The full `tools/pr_ready.py --quick` block for this head follows in a comment.

- New tests:
  - `sunrise-mod-package` `content::tests::fluids_declare_what_a_camera_inside_them_sees`: parse of all four fields. A fluid without `camera` has the default and `is_set` false. It refuses a non-id overlay, a colour above 0xFFFFFF, a distance set alone, an end not beyond its start, and an unknown key.
  - `sunrise-client-render` `fluid_models::tests::camera_looks_are_kept_by_liquid_block`: keyed by the named liquid block. Tank-only and unset looks are not kept.
  - `sunrise-client-render` `fog::tests::the_camera_is_in_a_mod_fluid_below_its_surface`: source and flowing heights, the same fluid above, another mod fluid above, and water.
  - `sunrise-client-render` `screen_effects::tests::a_mod_fluid_overlay_draws_its_texture_as_water_does`: same colour, UVs and world tint as water's quad, the fluid's own rect and sampling, and nothing for a texture the atlas lacks.
- Updated: the `content_golden` integration test's `fluid_full`/`fluid_minimal` goldens (the corpus `brine.json` now sets every `camera` field).
- Python checks run here without a build: `tests.test_mod_api_boundary`, `tests.test_recursive_wire`, `tests.test_mod_api_struct_kinds` (35 tests, OK), `tools/hotspots/check_lists.py`, `tools/hotspots/check_stand_ins.py`, `tools/ci/check_ignored_test_rows.py`. rustfmt --edition 2024 was run on every changed .rs file.

Tried in game: no.
