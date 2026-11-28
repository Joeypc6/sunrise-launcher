# Suggested update for #7115 (GTNH 2.9 port tracker), from Joeypc6's agent

> 🤖 Prepared by Joeypc6's AI agent (Claude Code), not by Joeypc6 personally. Checked against the issue as of 2026-11-28T15:20Z (snapshot `1b70e44bc09`). Totals are left to your audit re-run (section E).

## A. Engine gates

| Gate | Old | New | Reason and evidence |
|---|---|---|---|
| **G03** | `**READY**` | `**PARTIAL**` | In single player, mod block entities never tick on the integrated server. This has been true since the gameplay split [#6527](https://github.com/sunrise-org/sunrise/pull/6527) (merged 2026-11-27), so it holds at the snapshot `1b70e44bc09`. Filed by us as [#6984](https://github.com/sunrise-org/sunrise/issues/6984) (open). Cause: `prepare_mod_tickers` bakes only the preparing role's side, and the integrated server reuses that shared index. Dedicated servers are not affected. We run with a local fix (bake both sides); it is not upstream. Set back to READY when #6984 is fixed on main |
| G05 | `**READY**` | `**READY**` + note | Energy, fluids and items work for our ports (AE2, Powah, Ender IO, Tinkers fluids). Add the note: "64-bit energy and fluid queries and nested-transaction state are open in [#5456](https://github.com/sunrise-org/sunrise/issues/5456). GT5u's EU values are `long`." Make it PARTIAL only if darkliro counts cross-mod EU as part of the gate |
| G06 | `**PARTIAL**` | `**PARTIAL**` + note | Add: "the formed machine look (section geometry from block models) is [#1152](https://github.com/sunrise-org/sunrise/issues/1152), open. StructureLib is GPL-3.0, so a port needs its own checker or a GPL combined work" |
| G08 | `**READY**` | `**READY**` + note | Add: "vanilla tags losing entries with a mod's biome, structure or enchantment ([#6129](https://github.com/sunrise-org/sunrise/issues/6129)) is fixed and closed. Mod feature types are merged ([#1765](https://github.com/sunrise-org/sunrise/pull/1765)); worldgen placers [#953](https://github.com/sunrise-org/sunrise/issues/953) are still open" |
| G12 | `**READY**` | `**READY**` + note | Add: "connected textures [#5081](https://github.com/sunrise-org/sunrise/issues/5081), block appearance [#1106](https://github.com/sunrise-org/sunrise/issues/1106) and machine meshes [#1152](https://github.com/sunrise-org/sunrise/issues/1152) are open. Cables render today (AE2's cable bus), but GT-style layered machine faces and facades wait on these" |

The line under the gate table:

- Old: `` `PARTIAL` gates are the bottlenecks: G04 (recipe removal), G06 (multiblocks/StructureLib), G09 (custom gravity/portals), G11 (NEI/WAILA-class UI), G13 (1.7.10 coremods/Mixins/ASM). ``
- New: `` `PARTIAL` gates are the bottlenecks: G03 (single-player ticking, #6984), G04 (recipe removal), G06 (multiblocks/StructureLib), G09 (custom gravity/portals), G11 (NEI/WAILA-class UI), G13 (1.7.10 coremods/Mixins/ASM). ``

## B. "Already ported or covered" table

The existing rows match the registry (AE2 15.4.15, Tinkers 3.12.3, JEI 0.5.6, Jade 26.3.9, GuideME 0.1.1): no change.
Add a sub-table after it for ports in progress that aren't published yet:

```markdown
#### In progress (not yet in the registry)

| GTNH mod | Status | Sunrise work | Notes |
|---|---|---|---|
| GT5-Unofficial | `IN_PROGRESS` | Joeypc6, started 2026-11-28 | Work under way (licence audit, GregTech-size stress test on Sunrise); no released code yet. Steam age first |
| BloodMagic | `IN_PROGRESS` | Joeypc6 (port of Blood Magic 3.3.8, 1.20.1) | Not published. Modern version, not GTNH's 1.7.10 |
| EnderIO | `IN_PROGRESS` | Joeypc6 (port of Ender IO 5.3.73 from 1.12.2) | Not published. Not GTNH's 1.7.10 version |
| Draconic-Evolution | `PARTIAL_EQUIVALENT`, in progress | Sunny Evolution (Joeypc6, clean room) | Not published. Clean room because upstream's code is all rights reserved |
```

## C. Mod lines (old → new)

Each line keeps its gate and effort columns, as "How to use" asks.

1. **GT5-Unofficial** (Cross-Age)
   - Old: `- [ ] **GT5-Unofficial** — `BLOCKED_BY_GATE` — G01 G02 G03 G04 G05 G06 G07 G08 G09 G10 G11 G12 G13 G14 — LGPL-3.0 — XL`
   - New: `- [ ] **GT5-Unofficial** — `BLOCKED_BY_GATE` — G01 G02 G03 G04 G05 G06 G07 G08 G09 G10 G11 G12 G13 G14 — LGPL-3.0 (code; some textures CC BY-NC-SA 3.0 from UU-Tex) — XL — in progress: Joeypc6, started 2026-11-28 (steam age first; no released code yet)`
   - Why: our port started 2026-11-28; the first work is a GregTech-size stress test on Sunrise. No released code yet. The licence note is from GT5u's README, which credits Jimbno's [UU-Tex](https://github.com/Jimbno/UU-Tex) (CC BY-NC-SA 3.0) as the base of some textures.

2. **Railcraft** (Steam Age)
   - Old: `- [ ] **Railcraft** — `BLOCKED_BY_GATE` — G01 G02 G03 G05 G06 G07 G08 G10 G12 — NOASSERTION — XL`
   - New: `- [ ] **Railcraft** — `LICENSE_BLOCKED` — G01 G02 G03 G05 G06 G07 G08 G10 G12 — Railcraft licence (custom; API is MIT) — XL — no derived works without CovertJaguar's permission; clean room otherwise`
   - Why: [GTNewHorizons/Railcraft LICENSE.md](https://github.com/GTNewHorizons/Railcraft/blob/master/LICENSE.md) forbids derived jars, and works made from its code without explicit permission. GitHub's `NOASSERTION` hides that. Also note that GT5u has its own coke oven, so GTNH's core steam-age chain needs only Railcraft's coal coke, coke block and creosote items.

3. **Industrial Craft 2** (LV)
   - Old: `- [ ] **Industrial Craft 2** — `BLOCKED_BY_GATE` — G01 G02 G03 G05 G07 G10 G11 G12 — UNKNOWN — XL`
   - New: `- [ ] **Industrial Craft 2** — `LICENSE_BLOCKED` — G01 G02 G03 G05 G07 G10 G11 G12 — ARR — XL — clean room only; GT5u depends on it (rubber chain from the steam age)`
   - Why: IC2 experimental is all rights reserved (CurseForge page; GTNH's mod list). GT5u's `dependencies.gradle` has `api("net.industrial-craft:industrialcraft-2:2.2.828-experimental:dev")`, and the steam-age quests use IC2's rubber tree, treetap and sticky resin.

4. **Draconic-Evolution** (EV)
   - Old: `- [ ] **Draconic-Evolution** — `BLOCKED_BY_GATE` — G01 G02 G03 G05 G07 G09 G10 G12 — LGPL-3.0 — L`
   - New: `- [ ] **Draconic-Evolution** — `BLOCKED_BY_GATE` — G01 G02 G03 G05 G07 G09 G10 G12 — LGPL-3.0 on the GTNH fork, but upstream code ARR (CoFH "Don't Be a Jerk"), art CC BY-NC-SA 4.0 — L — clean-room equivalent in progress (Sunny Evolution)`
   - Why: GitHub reports LGPL-3.0 for the GTNH fork, but brandon3055's upstream code is all rights reserved. A port needs his permission or a clean room. Leave the verdict alone: reclassifying is darkliro's call.

5. **BloodMagic** (LV)
   - Old: `- [ ] **BloodMagic** — `BLOCKED_BY_GATE` — G01 G02 G03 G05 G06 G07 G10 G11 G12 — NOASSERTION — XL`
   - New: `- [ ] **BloodMagic** — `BLOCKED_BY_GATE` — G01 G02 G03 G05 G06 G07 G10 G11 G12 — CC-BY-4.0 (WayofTime/BloodMagic; GTNH's 1.7.10 fork not re-checked) — XL — in progress: Joeypc6 port of 3.3.8 (1.20.1), unpublished`
   - Why: our licence gate before starting found CC-BY-4.0 on GitHub for WayofTime/BloodMagic. Our port (3.3.8) is not published.

6. **EnderIO** (HV)
   - Old: `- [ ] **EnderIO** — `NEEDS_LICENSE_AUDIT` — G01 G02 G03 G05 G07 G10 G12 — NOASSERTION — XL`
   - New: `- [ ] **EnderIO** — `READY_TO_PORT` — G01 G02 G03 G05 G07 G10 G12 — Unlicense / CC0 (original code), LGPL-3.0-or-later (GTNH changes) — XL — in progress: Joeypc6 port of 5.3.73 (1.12.2), unpublished`
   - Why: checked 2026-11-28. [GTNewHorizons/EnderIO LICENSE.txt](https://github.com/GTNewHorizons/EnderIO/blob/master/LICENSE.txt) puts the original code in the public domain (Unlicense text; the README says CC0 1.0) and the GTNH modifications under LGPL v3.0 or later (COPYING, COPYING.LESSER). Bundled APIs from other mods keep their own licences. GitHub shows `NOASSERTION` only because the file combines two licences. Our port of the 1.12.2 version records the same public-domain grant in its NOTICE.

## D. Modpack glue

The questbook row stays as it is on #7115.

1. **GregTech ore veins and worldgen** row, "Sunrise route" cell
   - Add: "GT5u's veins are its own generator (`GTWorldgenerator`, `WorldgenGTOreLayer`, `WorldgenGTOreSmallPieces`), not biome-tag features. Being ported with GT5-Unofficial (in progress)."
   - Status stays **PARTIAL**.

## E. What the audit re-run will move

If C.2 and C.3 are accepted, Steam Age and LV each lose one `Blocked` and gain one `Licence`. If C.6 is accepted, HV
moves one from `Licence` to `Ready`. G03 becoming PARTIAL may push lines that list G03 from `READY_TO_PORT` to
`BLOCKED_BY_GATE`: in the Stone Age, StorageDrawers; in the Steam Age, OpenBlocks, Random-Things and SC2; in the MV
age, EnderStorage. That holds only until #6984 lands, so it may be better to keep G03 READY with a note. The choice
is darkliro's, since the gate's meaning is theirs.
