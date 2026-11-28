# Mod API (client): the sprites an item's default model draws with (item_sprites, like block_sprites)
> 🤖 Filed by Joeypc6's AI agent (Claude Code), not by Joeypc6.

## What a mod needs

To know which sprites an item's icon is drawn with, so it can read their pixels once per resource load and compute something from them. Example: a recipe viewer's colour search (JEI's `^red`) ranks every item by its dominant colours. Java JEI's `ColorGetter` takes the item's default `BakedModel`, collects the sprites its quads use (or its particle icon), and reads each sprite's pixels. Other uses: a minimap or sorting mod that groups items by colour, a particle effect in an item's colours.

The colour maths stays in the mod. Reading the pixels is #7160's half (below). What's missing is the mapping from an item to its sprites.

## What happens on main (8207f6d6aa)

- `block_sprites(&[BlockState])` answers a block model's faces, sprite names, tint indices and particle sprite. Nothing does the same for an item: an `ItemKey` has no read of the sprites its model uses.
- Pixels: #7160 asks for a decoded-image read of any `textures/...png` path over the client's merged resources, at any time. With the sprite names this issue asks for (`minecraft:item/apple` → `assets/minecraft/textures/item/apple.png`), that read covers the pixels. The existing `atlas_sprite_pixels(SpriteRef)` covers the blocks atlas only, and item textures aren't in it (`texture_sprites`: "a sprite only the items atlas lists is not in it").
- The only workaround is to open `assets/<ns>/items/<id>.json` with `open_resource`, then follow the item model's selectors, its model and the model's parents by hand to `layer0`/`layer1`. That repeats the engine's model resolution in every mod and breaks on any item model type the mod doesn't parse.

## Proposal

```rust
/// Published asset/data read (Setup, Reloading, World), like `block_sprites`.
/// The textures `item`'s default model draws with (`ItemModelResolver` on a default stack of
/// the item): each layer's sprite name (`minecraft:item/apple`) and tint index, in draw order,
/// plus its particle sprite. A block item gives its model's face sprites. None for an item the
/// game doesn't know. Read again when `block_sprites_version` (or an item equivalent) changes.
fn item_sprites(&self, items: &[ItemKey]) -> Vec<Option<ItemSprites>>;
```

`ItemSprites` mirrors `BlockSprites`, reduced to what an item model has: layers with sprite name and tint index, and the particle sprite.

One limitation of the path-based pixel read: a sprite that a sprite source generates (`paletted_permutations`, as armour trims use) has no PNG of its own. If that matters, `atlas_sprite_pixels` could also accept the items atlas. Ordinary item textures don't need it.

## Vanilla / NeoForge reference

- `ItemModelResolver.updateForTopItem` / `ItemStackRenderState`; a 26.x item definition (`assets/<ns>/items/*.json`) selects an `ItemModel` whose baked quads carry `TextureAtlasSprite`s, plus a particle icon.
- JEI's `ColorGetter.getColors(ItemStack)` reads the default model's sprites, not per-stack models.

## Related

- #7160 (client merged-view resource reads with decoded images): the pixel half.
- `block_sprites`, `texture_sprites`, `atlas_sprite_pixels`, `block_sprites_version` (API 380).
- #3147 (mod sprite sources for the blocks and items atlases).
