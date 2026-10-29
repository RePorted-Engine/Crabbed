# Launcher and engine lifetime

The native launcher starts with only the app window, settings and request dispatcher. Resource loading and the game renderer are deferred until Play. Quit Game from the title requests shutdown and returns to the launcher; the next Play constructs a fresh engine.

Shutdown stops gameplay callbacks and audio, parks presence/heartbeat activity, cancels pending world initialization once, and polls resource-pack and shader workers until they finish. It then releases the game, renderer, streaming pools and owned resource generation. Collision/motion tables use shared ownership instead of borrowing a permanently leaked resource container. Identical immutable block registries are reused across sessions; the existing static registry representation retains one registry per distinct content, rather than another duplicate on every Play/Quit cycle.

While the launcher is idle, the external display shows “Game is not running.” The display window and UIKit label remain available independently of the game renderer. On gameplay, the label hides and the renderer uses the display again.

## Validation

- iOS Clippy with warnings denied passed.
- Repository quick self-check passed: 372 Python tests, 1,713 Rust tests across the affected crates, formatting, Clippy, and five content-root tests.
- Native launcher preview: six UI and sixteen logic tests passed on iOS 27.
- Full iOS Simulator game: initial launcher had no engine control endpoint or resource-loader thread; two Play/title/Quit cycles recreated and removed the endpoint correctly. The process stayed alive in the launcher after Quit, with settled sampled CPU around 0.5%. These are simulator observations, not device performance guarantees.
- The simulator game used `HORIZON_FLAT_DRAWS=0`, an existing direct-draw fallback. Its Metal device rejects the indirect buffers in the default flat-draw path. That separate renderer limitation is not changed here.
- Physical AirPlay presentation and repeated-cycle device memory measurements remain to be checked on hardware.

Worker shutdown tests cover nonblocking polling/cancellation and completed-result disposal. Registry tests cover identical-session reuse and distinct default-state mappings.
