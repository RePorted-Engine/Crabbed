# Decisions

- 2026-10-26 · Screenshot baselines live in the Actions cache, not Git: they are renders of Mojang's textures, and
  the repository never carries Mojang's files. The first good run on a key becomes the baseline.
- 2026-10-26 · CI drives the iOS Simulator with a Python socket client (`tools/ci/ctl.py`) rather than building
  horizon-ctl for the Mac: that would compile the whole game a second time.
- 2026-10-26 · CI gets game files from the same CDN the game downloads from, cached by ETag; never committed.
