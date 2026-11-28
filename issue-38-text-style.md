> 🤖 Filed by Joeypc6's AI agent (Claude Code), not by Joeypc6.

## What a mod needs

To send chat and command feedback with a style: bold or coloured labels, and links the player clicks. Java mods build `Component.literal(..).withStyle(s -> s.withBold(true).withClickEvent(new ClickEvent.RunCommand("/..")).withHoverEvent(new HoverEvent.ShowText(..)))`.

Example: AE2's `/ae2 spatial` (Java's `SpatialStorageCommand`) lists the spatial storage plots. Each line has bold labels, and the player clicks it to teleport to the plot or show its details (a `RUN_COMMAND` click with a `SHOW_TEXT` hover). In the port, every line is plain text and nothing can be clicked.

## What happens on main (d11b3b9d8d)

- `crates/sunrise-mod-vocab/src/text/mod.rs` has everything the mod needs:
  - `pub struct TextStyle` (`#[non_exhaustive]`, every field `pub`: `color`, `bold`, `italic`, `underlined`, `strikethrough`, `obfuscated`, `click: Option<TextClick>`, `hover: Option<TextHover>`, `insertion`, `font`, ...), with `impl Default`.
  - `pub enum TextClick` (`RunCommand`, `SuggestCommand`, `OpenUrl`, `CopyToClipboard`, `ChangePage`, `ShowDialog`, `Custom`).
  - `Text::with_style(self, style: TextStyle)`, which is public.
- `crates/sunrise-mod-vocab/src/exports/text.rs`, what mods see, exports only `InlineImageReplacement`, `Text`, `TextHover` and `TextFormError`.
- So a mod can call `with_style`, and can name `TextHover::Text`, but can't name the `TextStyle` it takes or the `TextClick` it would put in it.

## Proposal

1. Add `TextStyle` and `TextClick` to `exports/text.rs`.
2. Because `TextStyle` is `#[non_exhaustive]`, a mod can't build it with a struct literal. `TextStyle::default()` followed by field assignment works, but a few builders make the common case one line, for example:

```rust
impl TextStyle {
    pub fn bold(mut self, on: bool) -> Self;
    pub fn color(mut self, color: impl Into<String>) -> Self;        // named or #RRGGBB
    pub fn click(mut self, click: TextClick) -> Self;
    pub fn hover(mut self, hover: TextHover) -> Self;
}
```

These match vanilla's `Style.withBold`, `withColor`, `withClickEvent` and `withHoverEvent`. Italic, underline, strikethrough, insertion and font can follow the same pattern.

## Vanilla reference

- `net.minecraft.network.chat.Style` (`withBold`, `withColor`, `withClickEvent`, `withHoverEvent`); `ClickEvent.RunCommand`, `HoverEvent.ShowText`; `MutableComponent.withStyle`.

## Related

- #1770 (scoreboard text, which uses the same `Text`).
