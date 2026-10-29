# Native-launcher integration review

The integration starts at the iOS fork's `f8319ef32` and merges the native-launcher branch through `697735d32`. It keeps the fork's pipeline/shader deep-stack fixes, event-handler stack growth, Mojang skin fallback and capes, title-screen account support on other platforms, and external-display/splitscreen support. The native launcher remains the account owner on iOS; the game's account button returns to it instead of creating a second independently persisted session.

This is a broad integration because the native branch also carries upstream engine history that was not incorporated into the fork's current tree. Its gameplay, resource and shader improvements are retained rather than replacing the branch with a launcher-only patch. Merge conflicts were compared by behavior; the prior direct-title integration is the reference for shared renderer, splitscreen and input changes.

The launcher has three native destinations: Play, Packs and Settings. Worlds and Servers remain available through Minecraft's menus. Its account control is circular, Microsoft profile names are read-only, and the option controls operate before game resources are installed. Pack mutations, searches, archive replacement and account handover guard asynchronous completion and surface failures.

No signing material, prepared resources or session credentials belong to this change. The native `HorizonMSAClientID` is empty in source; the iOS workflow fills it from the `HORIZON_MSA_CLIENT_ID` repository variable. A client ID already present in the fork's Rust Microsoft implementation is not changed here.

## Review and verification boundaries

GPT-6 Astra at medium effort reviewed the conflict resolutions, lifecycle, identity ownership, skin/cape recovery and publication-sensitive additions, and found no blocking introduced issue. The public-tree and history checks passed. The simulator preview uses dummy account/settings data; it does not exercise online authentication or Minecraft rendering.

Physical iPad testing before this integration confirmed touch/trackpad alignment, keyboard-to-touch switching, chat arrow keys, item descriptions, server-pack pointer release, view-bobbing shaders, invisible-stand equipment and affected NPC skins. Those results apply to the earlier device builds. The final integrated tree still needs a fresh physical multiplayer, shader and external-display regression pass.

Existing limitations include generic item/block display entity rendering, a cached Microsoft profile without its Keychain entry (see `launcher-bug-sweep.md`), and a shared thread-local shader parser slot across splitscreen renderers that can discard the other renderer's completed load. These are not newly resolved by this integration.
