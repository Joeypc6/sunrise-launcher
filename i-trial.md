> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

Vanilla 26.3's `TrialSpawner` creates any entity type its spawn data names, the way `BaseSpawner` does: it loads the entity with `EntityType.loadEntityRecursive`, runs the mob checks (`Mob.checkSpawnRules`, `checkSpawnObstruction`, `finalizeSpawn`) only when the entity is a `Mob`, and adds it with `addFreshEntityWithPassengers`. It then tracks the spawned entity by UUID in its `currentMobs` set until it dies or leaves.

Sunrise's trial spawner only spawns types from the mob table, and it tracks its spawns by mob UUID only, so a trial spawner whose spawn data names an armor stand, a boat, a minecart, primed TNT or the ender dragon spawns nothing.

Block and minecart spawners get the same fix in #6599, which leaves trial spawners out because their tracking needs its own change:
- spawn any entity type the level can create, as `BaseSpawner` does in #6599;
- track every spawned entity by UUID, not only mobs, so the trial's wave count and its "all spawned entities gone" check include them, as vanilla's `currentMobs` does;
- keep the mob path unchanged.

How to test: a trial spawner with an armor-stand spawn data spawns its wave of armor stands; the wave ends only when they're gone; a mob trial spawner behaves as before.
