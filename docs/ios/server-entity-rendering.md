# Server entity rendering regressions

Invisible armor stands hide their wooden base model, but continue to draw worn and held equipment. Servers can build decorations and NPC parts from those items. `armor_stand_geometry` previously returned before every equipment layer when the stand was invisible.

NPC player profiles remain usable while their entities exist after removal from the tab list. Texture and resolved-skin lookups are released when neither a player-list entry nor a level entity refers to them, with cleanup on entity-id reuse and dimension changes. Repeated profile additions replace stale texture properties; identical properties preserve an already resolved lookup. Pending fallback skin results may also finish for an unlisted entity still in the level.

Vanilla references inspected from the pinned 26.3 client bytecode:

- `LivingEntityRenderer.submit` skips the body when its render type is null but still calls the layers; `shouldRenderLayers` returns true.
- `AbstractClientPlayer.getPlayerInfo` caches the profile after its first lookup, independently of later player-list removal.

Validation:

- Game binary tests cover NPC profile replacement and lifetime with multiple entities and tab entries.
- `tools/armor_stand_render_e2e.py` renders an empty invisible stand, an equipped invisible stand, and an equipped visible stand on the real renderer in a fresh private world. It waits for camera/lighting settling and rejects a changing background before comparing equipment/body regions.
- The native item/block-display entity renderers are outside this change. The exact server-side representation of a particular Hypixel object must be confirmed by a live retest; these checks establish armor-stand equipment visibility and profile-cache behavior, not universal server-model compatibility.

Run the render check with built `horizon-client` and `horizon-ctl` binaries, the player's prepared resource path, and an output directory outside the repository. It uses isolated runtime homes, disables sound/self-update and cleans its temporary world on exit. It does not require Microsoft sign-in or connect to a public server.
