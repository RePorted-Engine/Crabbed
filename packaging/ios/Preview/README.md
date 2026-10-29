# Native launcher UI preview

This simulator-only app runs the production SwiftUI launcher with sample worlds/server data and a stateful settings backend. It does not launch the game, require Minecraft assets, or use a signing certificate. Its app bundle ID is separate from the game.

Generate the engine's option schema from the repository root:

```sh
cargo run --locked -p horizon-client --example native_launcher_options --no-default-features > packaging/ios/PreviewSupport/EngineOptions.json
```

Generate the preview project, then build/run `LauncherPreview` in Xcode with an iPhone or iPad simulator:

```sh
cd packaging/ios/Preview
xcodegen generate
```

The scheme includes settings model tests and UI tests for visible controls, editing a toggle, and retaining the change after tab navigation. `PREVIEW_OPTIONS_RESPONSE=empty` or `error` exercises failure/retry presentation; omitted, the backend reads `EngineOptions.json` and implements both `options-get` and `options-set`. These preview files are excluded from the game's launcher framework.

`PREVIEW_SIGNED_IN=1` seeds a fake profile for identity-control tests, `0` selects offline mode, and omission preserves the preview's existing profile. Play displays a preview notice instead of starting Minecraft. Use fresh scratch homes for the `HORIZON_*` and `XDG_*` variables when driving automated runs, as described in the repository's agent instructions.
