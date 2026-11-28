# Mod API (client): an item's sprites from its default model, and the pixels of a sprite outside the blocks atlas
> 🤖 Filed by Joeypc6's AI agent (Claude Code), not by Joeypc6.

## What a mod needs

To read the pixels of the sprites an item's icon is drawn with, once per resource load, and compute something from them. Example: a recipe viewer's colour search (JEI's `^red`) ranks every item by its dominant colours. Java JEI's `ColorGetter` takes the item's default `BakedModel`, collects the sprites its quads use (or its particle icon), and reads each sprite's `SpriteContents` pixels. Other uses: a map or minimap that colours items, a sorting mod that groups by colour, a particle effect in an item's colours.

The colour maths stays in the mod. What it can't get is the sprites and their pixels.

## What happens on main (8207f6d6aa)

- `RenderAssets::atlas_sprite_pixels(SpriteRef) -> Result<AtlasSpritePixels, AtlasSpriteError>` reads a sprite's first frame on the CPU. A `SpriteRef` names the blocks atlas only (`TextureAtlas::Blocks`).
- `ClientRenderClientHost::texture_sprites(&[&str])` resolves texture ids in the blocks atlas, and its docs say "a sprite only the items atlas lists is not in it". So `minecraft:item/apple` and a mod's `mymod:item/gear` aren't readable.
- Item textures drawn flat live in the GUI atlas (`TextureAtlas::Gui`, found with `gui_sprite`, which answers an `AtlasRegion`). Nothing reads its pixels.
- `block_sprites(&[BlockState])` answers a block model's faces, sprites and tint indices. Nothing does the same for an item: an `ItemKey` has no read of the sprites its model uses.
- The only workaround is to open `assets/<ns>/items/<id>.json` with `open_resource`, follow the model and its parents by hand to `layer0`/`layer1`, and decode the PNGs in the mod. That misses sprites a sprite source generates, and it repeats the engine's model resolution in every mod.

## Proposal

Two reads, both in the published-asset group (Setup, Reloading, World), like `block_sprites` and `atlas_sprite_pixels`:

```rust
/// The textures `item`'s default model draws with (`ItemModelResolver` on a default stack, no
/// components beyond the item's own): each layer's sprite id (`minecraft:item/apple`) and tint
/// index, in draw order, plus its particle sprite. A block item's model gives its block faces'
/// sprites. None for an item the game doesn't know. Changes on resource reload.
fn item_sprites(&self, items: &[ItemKey]) -> Vec<Option<ItemSprites>>;

/// The pixels of the sprite named `texture`, in whichever atlas the game stitched it (blocks,
/// items, GUI or a mod-registered one): the first frame, untinted, RGBA8, as
/// `atlas_sprite_pixels` answers. `NoSuchSprite` for a name no atlas holds.
fn sprite_pixels(&self, texture: &Identifier) -> Result<AtlasSpritePixels, AtlasSpriteError>;
```

`item_sprites` mirrors `block_sprites`. `sprite_pixels` widens the existing pixel read from a `SpriteRef` to a sprite name, so it covers the item and GUI atlases. A version counter (as with `block_sprites_version`) tells the mod when to read again.

## Vanilla / NeoForge reference

- `ItemModelResolver.updateForTopItem` / `ItemStackRenderState`; 26.x item models (`assets/<ns>/items/*.json`) select a `BlockStateModel`/`ItemModel` whose quads carry `TextureAtlasSprite`s.
- `TextureAtlasSprite.contents()` → `SpriteContents` (`getOriginalImage`, `isTransparent`). JEI's `ColorGetter.getColors(ItemStack)` reads those per sprite.

## Related

- `atlas_sprite_pixels`, `texture_sprites`, `block_sprites` (API 380), `gui_sprite`.
- #3147 (mod sprite sources for the blocks and items atlases).
