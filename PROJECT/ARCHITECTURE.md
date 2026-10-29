# Architecture (iOS port notes)

The game's own map is `CLAUDE.md` (crates, where each subsystem lives, the two single-player paths) and
`docs/architecture.md` (feature -> files -> harness). This page adds only what the iOS port changes.

| Piece | Where |
|---|---|
| Game entry on iOS (cwd = Documents) | `crates/horizon-client/src/main.rs` |
| Microsoft sign-in (title screen's face button, no launcher) | `msa.rs` (device code, Xbox Live, Minecraft; refresh token in the game home), `title.rs` `Mode::Account` |
| The phone's audio session (playback, mixes) | `crates/horizon-airplay/src/audio.rs`, from `main.rs` |
| Game files: download, resume, install | `import/download.rs` (ETag + If-Range resume), `import.rs` (`Pack::open/install`), `game/import_screen.rs` |
| Skins without an account | `skins.rs` (`ElySkins`), asked in `game/remote.rs` PlayerInfoUpdate |
| Shader packs (clean room) | crate `horizon-shaders`, `game/shader_packs.rs`, Options > Horizon Settings > Shader Packs |
| Xcode project | `packaging/ios/project.yml` (XcodeGen), `GameInfo.plist` |
| CI scenes | `tools/ci/screenshots.py` (horizon-ctl or `tools/ci/ctl.py`), `contact_sheet.py`, `screenshot_diff.py` |

Control protocol: one line in, one JSON line out over a Unix socket (`--control-socket PATH`); `wait-ready` is a
client-side poll of `status`.

Local builds here: `cargo build --profile fast -p horizon-client --no-default-features` (this machine has no ALSA
headers). Game files for local runs: `/tmp/iris-tools/cdnzip/unpacked/26.3` (the CDN archive, unpacked).
