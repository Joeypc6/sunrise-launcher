> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

Title: Mod API: item frame and armor stand entity templates with mod variants

A mod can't register its own item frame or armor stand: an entity type built on vanilla's `ItemFrame` / `ArmorStand` with a few overrides. NeoForge mods subclass them. Tinkers' Construct needs both (`tconstruct:fancy_item_frame`, `tconstruct:armor_stand`); neither #6131 (projectiles) nor #5174 (slimes) covers them.

**Item frame template** (vanilla `ItemFrame` behaviour, plus):
1. A synced, saved variant number, known at the first render (carried in the spawn data, as the hanging position and facing are).
2. A rotation count of 8 or 16, and a rotate hook (clamp instead of wrap, play a sound).
3. A server and client tick hook (frames that turn on their own; a timer that runs on both sides).
4. An interact pre-hook that can pass a right-click to the block behind (a clear frame).
5. The frame's own drop item and pick result (pick with an item in it still returns the item).
6. Fire and explosion immunity flags.
7. A comparator output override (16-step frames output the raw rotation 0..16).
8. A mod renderer per variant, including drawing no frame at all.
Setting the item on a turning frame resets its rotation, as vanilla does on rotation changes.

**Armor stand template** (vanilla `ArmorStand` behaviour, plus):
1. A synced, saved variant number and a left-handed flag.
2. Its own break-drop item (with the stand's custom name), keeping vanilla's equipment drops.
3. A fire-immune flag, a main-arm override, and an invisible-to-viewer override (a clear stand shows only its gear).
4. Setting vanilla's own flags (Small, ShowArms) from the mod.
5. A mod renderer per variant (for example full bright).
Placement follows vanilla's armor stand item (room for the box, yaw snapped to 45 degrees, place sound, `ENTITY_PLACE` game event, dispenser placement).

Both stay server-authoritative and add nothing for vanilla frames and stands. Tinkers' Construct 3.12.1 is the reference implementation (`FancyItemFrameEntity`, `FancyArmorStandEntity`).
