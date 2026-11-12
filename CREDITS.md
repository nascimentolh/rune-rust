# Original files used

## Read at run time (not copied here)

The game cache, folder set by `cache` in `server.cfg`: OpenRS2 cache #309 (build 727, matching the client;
https://archive.openrs2.org/caches/runescape/309/disk.zip), saved locally under `data/cache-727/` (not in git).
Earlier: OpenRS2 cache #302 (build 718):
`cache-runescape-live-en-b718-2012-05-29-00-00-00-openrs2#302/cache/` (from archive.openrs2.org)
- `main_file_cache.dat2`, `main_file_cache.idx0` … `idx36`, `main_file_cache.idx255` — served by the
  server; the client downloads index 33 (loading screens), 32 (their images and font glyphs),
  13 (font metrics) and 2 (configs) and draws the loading screens from them.

## Copied into this folder

- `data/keys-727.json` — map keys for cache #309, from https://archive.openrs2.org/caches/runescape/309/keys.json
- `data/keys.json` — map keys for cache #302 (build 718)
- `shots/loading-screen.png`, `shots/now.png` — self-test pictures that show cache art.

## Reference client code read and ported to Rust

- `com/jagex/EntityDefaults.java`, `Class348.java`, `CS2Interface.java`, `Class464.java`, `StructIndexLoader.java`, `RenderAnimIndexLoader.java`, `MovingAnimation.java`, `SkyboxIndexLoader.java` — login window id, interface loading
- `com/jagex/IComponentDefinitions.java`, `ComponentType.java`, `IFTargetParams.java` — interface component format
- `com/jagex/CS2Script.java`, `CS2Instruction.java` — CS2 script format and opcode table

Folder: the open727Client project's `src/main/java/`
- `com/Loader.java` — build 727.1, ports 5555, RSA and ISAAC switches
- `com/jagex/client.java` — draw routine per game state, fallback colours
- `com/jagex/IndexLoaders.java`, `LoadingStage.java`, `Preference_Sub20.java` — start-up stages, texts, percentages
- `com/jagex/Class306.java`, `Class165.java`, `NodeCollection.java` — loading-screen thread, crossfade, switching
- `com/jagex/Class304.java`, `Class158_Sub1_Sub2.java` — first (fallback) loading bar
- `com/jagex/GameTipsLoader.java`, `Class399.java`, `Class402.java`, `Class393.java`, `Class112.java`, `Class60.java` — index 33 format
- `com/jagex/Class299.java`, `Class29.java`, `Class42.java`, `Class51.java`, `Class52.java`, `Class52_Sub2.java`, `Class52_Sub2_Sub1.java` — element drawing
- `com/jagex/Class357.java`, `Class62.java`, `Class350.java`, `Class350_Sub3.java`, `Class350_Sub3_Sub1.java`, `Class383.java`, `Class258.java`, `Class163.java`, `Class332.java`, `RouteStrategies.java`, `CutsceneActionType.java`, `Class366.java`, `IFSubObjectPosition.java`, `MapRegion.java`, `VarDefinitionLoader.java`, `Class214.java`, `Class356.java`, `Class353.java` — element decoders and anchors
- `com/jagex/SpriteDefinitions.java`, `Class152.java`, `Class103_Sub1.java`, `Class395.java` — sprite and image decoding
- `com/jagex/FontMetrics.java`, `FontRenderer.java`, `FontRenderer_Sub1.java`, `FontRenderer_Sub3.java`, `JavaRenderer.java`, `Utils.java` — fonts, kerning, text layout
- `com/jagex/NativeSprite.java`, `NativeSprite_Sub1.java` — sprite tiling
- `com/jagex/ByteBuf.java`, `LocalizedText.java`, `Preference_Sub11.java`, `CS2ReturnValue.java` — number readers, texts, defaults, animation clock
