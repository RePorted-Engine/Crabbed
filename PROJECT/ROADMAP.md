# Horizon feature gap roadmap

2026-10-26. Built from: docs/known-issues.md, docs/rust-parity.md, docs/server-levels.md, packaging/ios/README.md, marker greps over crates/horizon-client/src and crates/horizon-world/src (few real markers: 1 "not wired", 5 "not implemented", 2 "TODO", 0 `todo!`/`unimplemented!`), and the 60 newest open issues of ludovic111/horizon (titles only). Impact H/M/L, effort S/M/L are estimates. Each section is sorted by impact, then effort.

## World & terrain
- [ ] Amplified world type does not exist — impact M, effort M — issue "Amplified world type doesn't exist"
- [ ] Superflat cannot be customized — impact M, effort M — issue "Can't customize superflat"
- [ ] Debug world not accessible — impact L, effort S — issue "Can't access minecraft debug world"
- [ ] Structure types with no generator fall back to `Unsupported` — impact M, effort L — crates/horizon-world/src/worldgen/structure/start.rs:101
- [ ] Worldgen roots skipped when algorithm/bindings missing (reported list) — impact M, effort L — crates/horizon-world/src/worldgen/generator.rs:2269, generator_decoration.rs:4
- [ ] Value providers not all covered; inline noise holders unsupported in density — impact L, effort M — worldgen/value_provider.rs:2, worldgen/density.rs:68
- [ ] Distant LODs wrong with floating blocks — impact L, effort M — issue "Distant LOD's are not correct with floating blocks"
- [ ] Chunk loading causes large lag spikes — impact M, effort L — issue "Massive lag spike from chunk loading..."

## Blocks & items
- [ ] Bundle UI never shows up — impact M, effort S — issue "Bundle UI never shows up"
- [ ] Cannot place food on campfire — impact M, effort S — issue "Can't place food on campfire"
- [ ] Cannot place cushions — impact L, effort S — issue "Cant place cushions"
- [ ] Water does not carry items — impact M, effort S — issue "Water does not carry items"
- [~] Piston-pushed blocks teleport (fixed 2026-10-26 for server-thread/multiplayer clients); observer clock breaks piston (open) — impact M, effort M — issues "Piston pushed blocks teleport", "observer clock makes piston bug"
- [x] Potions become "uncraftable potion" in inventory / as trade item (fixed 2026-10-26: potion_contents carried both ways) — impact M, effort M — issues "All potions transform into uncraftable potion...", "Uncraftable potion as trade item"
- [ ] Recipe book: list not selectable, 3x3 recipe drawn in 2x2 grid, overlays persist — impact M, effort M — issues "Recipe book: recipe list cannot be selected...", "recipe book overlays stay visible..."
- [ ] Block-break textures wrong and jitter — impact L, effort M — issues "Pretty much every break texture...", "Block breaking texture moves a few pixels..."
- [ ] Ghost items in crafting grid after closing a crafting table; shift-click in chests stops working — impact M, effort M — issues "Ghost blocks/items appear...", "After using Shift + Right-Click..."
- [ ] Hotbar toolbar screen (save/load keys) not wired to HotbarManager — impact L, effort M — docs/rust-parity.md:1454

## Entities & mobs
- [~] No ender dragon (upstream PR #2049); glowing effect does not work (fixed 2026-10-26: outlines drawn, team colours) — impact H, effort M — issue "No ender dragon and glowing effect does not work"
- [x] Skeletons do not aim bows at players — verified 2026-10-26: bow pose raised and drawn (fixed by 3e9fe5a8, the aggressive flag) — impact H, effort S — issue "Skeletons don't aim bows at players when shooting"
- [x] Peaceful difficulty does nothing — verified 2026-10-26: /difficulty peaceful removes zombies, health 14→20 in 100 ticks — impact H, effort S — issue "peaceful doesn't do anything" (in progress)
- [ ] Villagers do not take a profession from player-placed workstations — impact M, effort M — issue "Villagers don't get their profession..."
- [ ] Projectiles and block entities do not visually update until they land; thrown items lag — impact M, effort M — issues "Projectiles and block entities don't visually update...", "throwable stuff lag/stay in place..."
- [ ] Mobs/items/minecarts entering portals between server levels, and portal tickets, unhandled — impact M, effort L — docs/server-levels.md:95
- [ ] Happy ghast dismount drops player through it — impact L, effort S — issue "[server-thread] Dismounting a happy ghast..."
- [ ] Bat wings, turtle feet, squid direction, zombie-horse double riding, boat player animation — impact L, effort S — issues "Bats don't flap wings", "Weird turtle feet behavior", "Squid direction bug", "riding a zombie horse...", "Wrong player animation at boat"
- [ ] Raider goals, banner fetching, ravager riding not compared against vanilla; village distance not from PoiManager — impact L, effort L — docs/rust-parity.md:1577-1587

## Player mechanics
- [x] Creative players can break blocks holding a sword (fixed 2026-10-26, the tool component's can_destroy_blocks_in_creative) — impact M, effort S — issue "Creative players can break blocks while holding a sword"
- [ ] Sprint-flying with Control increases speed — impact L, effort S — issue "When flying in sprint, holding control increases speed"
- [ ] Swimming-upward collision hitbox wrong — impact L, effort S — issue "Collision hitbox wrong when swimming upwards"
- [ ] Drowning/air supply not tracked (advancement `air` criterion stays 300) — impact M, effort M — docs/rust-parity.md:1374
- [ ] Damage clock: enclosing ticks incl. death removal not bound — impact L, effort M — crates/horizon-client/src/health.rs:242
- [ ] Critical-hit particles missing — impact L, effort S — issue "Particles for critical hits missing" (in progress)
- [ ] End credits for server players (first End exit, seenCredits) — impact L, effort M — docs/server-levels.md:94
- [ ] Narrator, chat delay, distortion effects, glint speed/strength, monochrome logo, minecart rotation, high-contrast outlines: options shown disabled — impact L, effort M — crates/horizon-client/src/options_ui.rs:163, 2223

## Game loop & commands
- [x] `/place structure` fails: Structure.generate not wired to the live level — impact M, effort M — crates/horizon-client/src/game/commands/place.rs:180 (done 2026-10-26)
- [ ] `@e` without distance/volume reads only the source's level (vanilla reads every level) — impact M, effort M — docs/server-levels.md:98
- [ ] Shared LevelData not swapped per level (game time, weather, rules, scoreboard, maps) — impact L, effort L — docs/server-levels.md:24
- [ ] Display name components keep team formatting but not click/hover/insertion — impact L, effort M — docs/rust-parity.md:1375
- [ ] Mouse cursor not centred on opening inventory — impact L, effort S — issue "[Bug] Mouse cursor does not center on screen when opening inventory"
- [ ] Input and audio lag while multitasking — impact L, effort M — issue "Input and Audio Lag while multitasking"
- [ ] Poor performance in containers / creative search — impact M, effort M — issue "Poor performance in containers and specifically creative/search"

## Networking
- [x] Bossbar does not vanish properly (fixed 2026-10-26: reset on reconfiguration) — impact M, effort S — issue "Bossbar does not vanish properly"
- [ ] macOS integrated server panics on broken stdout pipe; world creation hangs — impact M, effort S — issue "macOS: integrated server panics with 'failed printing to stdout'..."
- [ ] Benchmark counts scene loaded before first chunk is sent — impact L, effort S — issue "`--benchmark`: a scene counts as loaded..."
- [ ] Playing ping should be optional — impact L, effort S — issue "Playing ping made optional, but on by default"
- [ ] Vanilla interop/reconnect coverage listed as "Not implemented" in the early parity table (likely stale; verify) — impact L, effort S — docs/rust-parity.md:344

## Persistence
- [ ] Chunks with a different DataVersion are rejected; datafixing absent in `horizon-world::chunk` — impact H, effort L — crates/horizon-world/src/chunk.rs:234 (a `datafix/` module exists; confirm it is wired for older worlds)
- [ ] Broken ender portal in the End bug — impact L, effort S — issue "bug when i broke ender portal in end"
- [ ] Windows first launch after Defender update takes 77-92 s (23,000 loose files) — impact M, effort L — issue "Windows: the first launch after a Defender signature update..."
- [ ] Prepare a version without Java on the player's machine — impact M, effort L — issue "Preparing a version without Java..."

## Rendering & audio
- [ ] Shader packs removed; to be rewritten — impact M, effort L — docs/known-issues.md:6
- [x] No title-screen background since v0.2.57 (not reproduced 2026-10-26: CI title scene shows the panorama) — impact M, effort S — issue "No Background on Title Screen after V0.2.57"
- [x] Continuous stone breaking sound (fixed 2026-10-26: no digging air after an instant break) — impact M, effort S — issue "Continuous stone breaking sound"
- [ ] Crash loading UI resource pack — impact M, effort M — issue "Crashed when loading UI resource Pack"
- [ ] End crystal explosion shows no effect on macOS; death-screen tilt glitch; arrow-like particles in water; far kelp flicker — impact L, effort M — issues of those titles
- [ ] Chinese text stops rendering after a stutter — impact M, effort M — issue "Chinese text stops rendering..."
- [ ] Translations: client's own screens stay English; other languages regressed — impact M, effort M — issues "Language: the client's own screens stay English", "[regression] Support other Languages"
- [ ] Discord Rich Presence missing on macOS — impact L, effort S — issue "Horizon Discord Rich Presence does not show up..."

## iOS & UI
- [ ] Touch input layer (winit loop is cursor-warp mouse look, no touch) — impact H, effort L — packaging/ios/README.md:54 ("A touch layer, in no crate yet")
- [ ] Assets: no Java on iOS; prepare off-device and download to the phone (download/resume work exists in git log) — impact H, effort M — packaging/ios/README.md:53
- [ ] Bundle is the probe crate, not the game; game .ipa is built with `--no-default-features --features audio` and untested on device — impact H, effort L — packaging/ios/README.md:45, 66
- [ ] Audio session setup for CoreAudio/RemoteIO on the phone — impact M, effort S — packaging/ios/README.md:56
- [ ] Controllers via GCController instead of hidapi/gilrs — impact L, effort M — packaging/ios/README.md:56
- [ ] Mods disabled on iOS (wasmtime JIT forbidden by Apple) — impact L, effort L — packaging/ios/README.md:54
- [ ] Phone-shaped HUD/menus (inventory, chat, on-screen keyboard) — impact H, effort L — inferred from the touch gap above; `osk.rs` exists (crates/horizon-client/src/osk.rs)
- [ ] Platform builds requested upstream: Intel Mac, Linux ARM64 — impact L, effort M — issues "Add Intel Mac Support", "Please provide a Linux ARM64 version"
