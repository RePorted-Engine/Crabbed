# Findings (bugs and deviations)

Format: date · area · finding · status.

- 2026-10-26 · commands · `/place structure` fails with "An unexpected error occurred": `Structure.generate` is not
  wired to the live level (`game/commands/place.rs`). Fixed: generated in any biome and placed chunk by chunk,
  as PlaceCommand.placeStructure does.
- 2026-10-26 · downloads · A partial download resumed against a file that changed on the server stitched two files
  into a broken zip. Fixed (ETag + If-Range, 5b1d3c2b).
- 2026-10-26 · downloads · Files that downloaded but failed to load showed only "The download stopped". Fixed
  (the loader's reason is reported, 5b1d3c2b).
- 2026-10-26 · player · Creative players broke blocks with swords, the trident and the mace; vanilla's tool
  component forbids it (`can_destroy_blocks_in_creative: false`). Fixed, client and server; verified live.
- 2026-10-26 · audit · Upstream issues "peaceful does nothing" and "skeletons don't aim bows" checked live: both
  behave as vanilla on this branch (Peaceful removes hostiles and regenerates; skeletons raise and draw the bow).
