> 🤖 Opened by Joeypc6's AI agent (Claude Code, Claude Opus 5.5), not by Joeypc6 personally.

On Windows, the first start after any change to the installed mods spends most of its time in `mods.overlay`. The game builds a new content root under `.sunrise-cache\mod-content\<hash>\`: `build_with_conditions` (`crates/sunrise-sim/src/mod_content/load_conditions.rs:190`) calls `Overlay::new` (`crates/sunrise-sim/src/mod_content.rs:693`). On `not(unix)`, `link()` (`mod_content.rs:770-784`) recurses into every directory of the prepared base and hard-links each file, falling back to `fs::copy`. On Unix the same `link()` makes one symlink per top-level entry.

Measured on Windows 11 with a 26.3 base of 28,674 files (659 MiB):
- Hard-linking the base the way `link()` does, outside the game: 43.7 s (1.55 ms per link). `remove_dir_all` of the result: 12.7 s.
- Start to title with a 1-item mod, cold: 29.5 s, against 7.7 s with no mod. With a 2,458-entry mod: `mods.overlay` alone was 51.5 s.
- The same mod on a warm start (same content set): `mods.overlay` 2.6 s. The cost is per base file, not per mod item.

Proposed fix: on Windows, link directories as NTFS junctions, which need no privilege (unlike `symlink_dir`), and fall back to the per-file links only when a junction can't be made. `Overlay::dir()` already turns a linked directory into a real one before writing under it: it must remove a junction with `fs::remove_dir`, not `fs::remove_file`. Check that removing an old root never follows a junction into the base. Expected: the cold overlay drops to a few seconds, about the warm figure plus the table rebuild.

How to test: a Windows test that builds an overlay over a small base, checks the top-level directories are junctions, writes a file under one (turning it real) and checks the base's copy is untouched, removes the root and checks the base is intact, and forces the fallback. Plus a before/after cold-start timing.
