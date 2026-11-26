> 🤖 Agent: posted for Joeypc6 (Claude Code, Claude Opus 5.5), not Joeypc6 personally.

Thanks, Dyna. All three points are addressed in one new commit on top of `62a5cf71` (nothing rewritten): `045c8f5f`. Line numbers are at that commit.

**Point 2: the ordinary build took a legacy reduced image at the full tag.** The planned path no longer trusts the `.installed` marker alone. The version a client plan installs as now comes from `reuse::installable` (`src/native_mods/reuse.rs:290`, used at `src/native_mods.rs:1892`). It uses the build under the plan's version only when that build holds the plan (`reuse::holds`, reuse.rs:275): `native-mods.json`, Play's `native_status::included` set and the installed mod copies have to match the plan (`Contents::installed_in`), and its game has to match its seal (point 1).

A build that doesn't hold the plan isn't replaced or deleted, because a profile may still point at it or run it. The plan installs under the next version instead (`reuse::versions`, reuse.rs:251). The next version is derived from the plan's own version, so it is the same every time for the same plan, and it is never the reduced build's version. The fast lookup walks the same list (reuse.rs:523). After 8 builds that all fail to hold the plan, the build stops with a message saying to remove them.

So the old reduced image is neither said to be up to date nor pinned. It stays as it is, unsealed, until clean-up removes it once nothing uses it.

Tests:
- `a_reduced_build_never_satisfies_the_whole_set` (tests.rs:490) now seeds the old reduced image at the full tag. The whole-set plan installs under the next version. The old record is unchanged and gets no evidence, and the same next version comes back every time.
- New: `the_default_build_never_takes_a_reduced_or_changed_build` (tests.rs:825) goes through ordinary `build`, in a scratch home, on a developer base. A developer base plans without an SDK check, so the planned path does reach the installed build. With the legacy image seeded at the full tag, `build` doesn't say `Plan(UpToDate)` and doesn't pin the instance. It goes on to build the next version, which the fixture can't compile, so it ends in an error.
- Against `62a5cf71`'s marker-only check (patched back in for one run in our build queue), that test fails at its first assertion: `build` returned the reduced image as built.

**Point 1: a slow-path miss could bless the executable it rejected.** A build is now sealed once: when it is published, or on first use if it has no evidence (an older launcher's build). `record` (reuse.rs:348) leaves evidence alone when its seal no longer matches the game or when it can't be read (reuse.rs:359): it doesn't overwrite it and doesn't re-seal. `holds` treats such a build as not installed, so the plan builds under the next version.

Torn, unknown-layout and unknown-field evidence are handled the same way: never re-sealed. Only a build with no evidence at all is adopted.

Tests:
- `an_installed_build_that_changed_misses` (tests.rs:596) now changes the executable after publication and plans again. The plan builds under the next version, the old evidence is byte for byte the same, and the next lookup finds the new build, not the changed one.
- The through-`build` test above does the same with the executable changed after a fast hit. Two ordinary builds in a row neither say `UpToDate` nor change the evidence.
- `missing_torn_or_older_evidence_plans_the_build` (tests.rs:540) covers torn, other-schema and unknown-field evidence: none is sealed again. It also covers missing evidence and the old `.native-inputs` file, which are sealed on first use.

**Security: `SUNRISE_SDK_BASE` in the evidence.** The evidence now keeps only a SHA-256 fingerprint of the setting, with a domain tag, so a different setting still misses (`sdk_base_fingerprint`, reuse.rs:133, applied at reuse.rs:126). This is the same comparison the `98b2edc7b8` key made by hashing it. The raw value is never written.

`the_evidence_never_holds_the_sdk_base_setting` (tests.rs:687) plans with `https://reader:s3cret-token@mirror.example/sdk`. It checks that the evidence file contains neither the setting nor any part of it, and that the fingerprints of different settings differ.

Checked on Windows 11 (Rust 1.99.1, x86_64-pc-windows-gnu) at `045c8f5f`:
- rustfmt on the changed files
- `cargo clippy --all-targets --locked -- -D warnings`: clean
- `cargo test --locked --lib native_mods`: 134 passed
- `structure_check.py --base origin/main`: every count +0

The new message ("Every installed build named for these mods…") went into every catalog with `tools/launcher_add_lang_keys.py`. macOS and Linux are not checked.
