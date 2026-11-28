> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

Two projectile kinds a mod can't register today, both outside #6131 (arrows, the trident-like thrown item, the fishing hook). Tinkers' Construct needs them for `tconstruct:fluid_spit` and `tconstruct:thrown_shuriken`.

**Plain projectile, no item** (vanilla `LlamaSpit`-like, for `fluid_spit`, `FluidEffectProjectile extends Projectile`):
1. Settable gravity, including its sign (Tinkers: 0.06 down, or 0.06 up for a fluid lighter than air), and drag (0.99 in air, a per-entity water inertia in water, capped at 0.99).
2. Synced, saved mod data (the carried fluid and amount; power; knockback), through the existing entity attachments.
3. On an entity hit: the mod decides whether it is discarded, so it can pass through while it has fluid left (by default it keeps flying). On a block hit: the mod decides, and can keep it flying through the hit point.
4. A client spawn hook for its particles (LlamaSpit's spit burst).
5. An owner that is an entity or a block position (a fluid cannon), with the owner block's inventory reachable by position.
6. Discard 64 blocks above the world.

**Thrown item that carries its stack** (`ThrowableItemProjectile`-like, for `thrown_shuriken`, `ThrownShuriken extends Projectile`), on main's thrown template:
1. A carried item stack, synced and saved, drawn as the projectile.
2. Per-entity water inertia (default 0.8), and settable power (flat damage) and knockback.
3. On an entity hit: the mod cancels vanilla's damage and deals its own damage and knockback; a pre-hit hook can disable the target's shield.
4. On a hit: drop the carried stack as an item entity where it lands (the "the stack comes back" case), or show break particles; then discard.
5. Nether portals and end gateways behave as for vanilla throwables (enter or teleport, and skip the hit).

Both stay server-authoritative and add nothing for vanilla projectiles. The hit stays cancellable through the existing `projectile_impact` hook.
