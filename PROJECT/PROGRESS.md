# Progress

- 2026-10-26 02:50 · Branch `build/2026-10-26`. CI: `linux.yml` (build + lavapipe screenshots + baseline diff),
  `ios-sim.yml` (simulator build, scenes, video), artifact scripts, `CI.md`. Screenshot driver tested locally
  (6 scenes). Download resume fix and Ely.by skins carried on the branch.
- 2026-10-26 03:55 · CI: linux.yml green on first run (6 scenes on lavapipe, artifacts fetched to ci-artifacts/latest/linux);
  ios-sim builds the simulator game and boots a simulator (first runs cancelled by newer pushes; now push runs finish).
  Shipped: `/place structure` (43aa3d0a), creative sword/trident/mace can't break (7c95b803), boss bars cleared on
  proxy reconfiguration, no stone knocking after instant breaks (e221254f). Verified live, no change needed: Peaceful,
  skeleton bow aim, title background.
