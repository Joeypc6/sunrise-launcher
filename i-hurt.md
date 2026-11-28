> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

A mod can't register a projectile built on vanilla's accelerating "hurting" projectile (`AbstractHurtingProjectile` / `Fireball`, as `small_fireball` is). #6131 covers arrows, a trident-like thrown item and the fishing hook; this kind is outside it. Tinkers' Construct needs it for `tconstruct:fireball` (`CustomFireball extends Fireball`).

A template on vanilla's small fireball behaviour (motion with acceleration power, drag, the display item), plus:
1. A carried display item, synced and saved, and named after it (the fired ammo sets it).
2. Settable "not deflectable": not pickable and not hurt, so it can't be punched or deflected back.
3. Settable "no burn": the projectile itself isn't on fire.
4. On hit: no vanilla fire and no explosion. The mod deals its own damage with its own damage type (Tinkers: `ceil(speed x power x multiplier)`, with enchantment damage effects for a living owner), and the projectile is discarded after any hit.
5. A maximum age (Tinkers: 2400 ticks), and discard when it goes 64 blocks below or above the world.
6. Saved per-entity values (damage power, multiplier, damage type ids) through the existing entity attachments.

Server-authoritative, no cost for vanilla fireballs. The hit stays cancellable through the existing `projectile_impact` hook, as for other projectiles.
