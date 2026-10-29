# CI

Every push to any branch runs these (docs-only pushes skip the builds). All have `workflow_dispatch`:
`gh workflow run <file> --ref <branch>`.

| Workflow | Runner | What it does | Artifacts |
|---|---|---|---|
| `ci.yml` (checks) | ubuntu | `cargo fmt --check`, clippy `-D warnings` (with and without default features), every lib/bin/test target, doc tests, the public-tree check, Python tests. Parity harnesses need the maintainer's runner. | none |
| `linux.yml` | ubuntu | `build`: `--profile fast` build of `horizon-client` + `horizon-ctl` (mold linker). `screenshots`: the scenes of `tools/ci/screenshots.py` drawn by lavapipe under xvfb, a contact sheet, and a diff against the baseline (the first good run's captures, kept in the Actions cache under `screenshot-baseline-linux-v1`; bump the key for a new baseline). | `linux-binary`, `linux-screenshots` |
| `ios-sim.yml` | macos-15 | The game cross-compiled for `aarch64-apple-ios-sim`, bundled, installed on a fresh simulator with the game files in Documents, driven over its control socket by `tools/ci/ctl.py`; one `simctl io screenshot` per scene, plus a video. | `ios-simulator-app`, `ios-screenshots` |
| `ios.yml` | macos | Per-crate iOS portability table, the unsigned Xcode shell `.ipa`, the unsigned game `.ipa` with and without the SwiftUI launcher. | `horizon-ios-game-with-launcher`, `horizon-ios-game`, `horizon-ios-unsigned-ipa`, … |

## Scenes
`title`, `spawn_day`, `structure` (the seed's nearest village, via `/locate`), `inventory`, `cave` (carved at y -40, torches and lava), `night_rain`. Seed 1234567, world `CiScenes`, time and weather frozen by gamerules, landscape scenes as a spectator. Run them locally:

```sh
python3 tools/ci/screenshots.py --game target/fast/horizon-client --ctl target/fast/horizon-ctl \
  --resources .local/horizon-client/26.3 --out /tmp/shots
python3 tools/ci/contact_sheet.py /tmp/shots     # needs Pillow
```

## Game files
Never in Git. CI downloads the archive the game itself downloads (`https://cdn.gayfeet.lol/horizon/26.3.zip`), cached by its ETag.

## Artifacts on your machine
```sh
scripts/fetch-artifacts.sh            # this branch's newest green linux + ios-sim runs -> ci-artifacts/latest/<workflow>
scripts/fetch-artifacts.sh main ios   # another branch / workflow
scripts/open-screenshots.sh
```
Windows: `.\scripts\fetch-artifacts.ps1 -Branch build/2026-10-26`.

## Signing
Nothing here signs. A signed, installable `.ipa` needs an Apple Developer certificate (`.p12` + password) and a provisioning profile as repository secrets, imported into a temporary keychain before `xcodebuild archive`/`-exportArchive`. Sideloaders (AltStore, Sideloadly) sign the unsigned `.ipa` instead.

## Speed
Rust caches (`Swatinem/rust-cache`) are saved on every push, per workflow (`linux-fast`, `ios-sim`, `verify`), so a branch's second run is warm; `CARGO_INCREMENTAL=0`; mold on Linux; game files cached by ETag; `concurrency` cancels superseded runs.

## Timings
Filled in from real runs below.

| Job | Cold | Warm |
|---|---|---|
