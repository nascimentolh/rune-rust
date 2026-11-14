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

## Reference server code read (not copied)

A build-727 private server (Kotlin/Java) at `C:/Users/User/Documents/Runescape/727-references/server-dev`,
read for what a server sends this client. It is somebody else's server, not Jagex's, so it is
evidence about the protocol (its encoders are written against the same client), not authority
about the game's rules; what this server chooses for itself is still its own.

- `core/.../net/prot/revision/rev727/Rev727ServerCodecsMisc.kt` — the `WORLD_LIST` payload: the
  flag bits (members 0x1, quick chat 0x2, bots allowed 0x4, LootShare 0x8, highlighted 0x10,
  custom cache 0x20, custom port 0x40000000) and the port written after a world's host; the
  world-login reply's field order
- `core/.../worldlist/WorldMetadata.kt` — what a world is given (number, host, port, activity,
  country, quick chat, LootShare, members, bots, highlighted)
- `world/.../player/managers/InterfaceManager.kt` — which interface goes in which slot of the
  game windows 548 and 746 (`Sub`: the tabs, the orbs, the logout tab, the chat frame 752, the
  filter bar 751 and the chat box 137 in 752:9) and the order the chat is opened in
- `world/.../player/Appearance.kt` — a new player's default male look (body styles and colours)
- `world/.../player/Skills.java` — the combat level formula (`getCombatLevel`)
- `world/.../npcs/PKBotNPC.kt` — base animations 1426 without a weapon
- `core/.../net/prot/revision/rev727/Rev727AppearanceEncoder.kt` — the appearance flag 0x1 means
  "not male"; body styles sent as 0x100 + the style
- `core/.../net/prot/revision/rev727/Rev727ServerCodecsMessaging.kt` — `MESSAGE_PUBLIC`'s layout
  and its 210-character cut
- `world/.../player/Player.java` (`sendPublicChatMessage`), `world/.../packet/ChatHandler.kt`,
  `ChatTypeHandler.kt` — who gets public chat, which lines aren't chat, and that `CHAT_TYPE` is
  kept for the next `CHAT`
- `world/.../pathfinder/RouteFinder.kt` (`routeEntityWalkRequest`) — the server routes a walk
  request over its collision flags
- `world/.../packet/WalkHandler.kt`, `world/.../entity/Entity.java` (`addWalkSteps`,
  `processMovement`, `needMapUpdate`), `world/.../pathfinder/RouteFinder.kt`,
  `core/.../Utils.kt` (walking and running directions), `world/.../update/EntitySnapshotFactory.kt`,
  `core/.../rev727/Rev727PlayerMasks.kt` (mask 0x4) — walking: the request, the steps, how many a
  tick, and the movement type sent to a new player

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
- `com/jagex/Huffman.java`, `LinkedNodeList.java` (`method7885`), `ByteBuf.writeHuffmanString` — the chat's text compression
- `com/jagex/PlayerUpdate.java`, `NewsItem.java` (`method1806`), `Class399.java` (`decodeSkip`), `Class197.java` (`method3203`) — the player update: lists, skips, the player staying, mask blocks
- `com/jagex/PlayerEntity.java` (`decodeAppearance`, `getUsernameWithTitle`), `EquipmentDefaults.java`, `DefaultsFile.java` — the appearance and the equipment slot table
- `com/jagex/PacketDecoder.java` (`MESSAGE_PUBLIC`), `Node_Sub33.java` (`method12582`), `Class182.java` (`method3044`), `Class76.java` (`getCrown`) — public chat from the server
- `com/jagex/CutsceneArea.java` (`method12078`), `PathingEntity.java`, `MapRegion.java` (the scene base) — walking: `WALK`, the path, the scene's base
- `com/jagex/SystemInfo.java` (`method13466`), `Class363.java`, `Class155.java` (`method2636`), `Class19.java`, `MapSpriteIndexLoader.java` (`method7188`), `BASDefinitions.java`, `Class190.java`, `Class501.java` — moving along the path, turning, base animation sets, the camera following

- `com/jagex/GraphNode_Sub1_Sub4_Sub1.java` (`iComponentOnGroundTile`), `Class4.java` (`method288`), `JavaRenderer.java` (`method8454`) — "Walk here" on the ground tile under the mouse

- `com/jagex/ClipFlagMap.java`, `ClipFlag.java`, `Class329.java` (`decodeTileMasksUnderlay`, `initClipMap`), `Class329_Sub1.java` (`method12459`, `method12464`), `LocationIndexLoader.java`, `RouteFinder.java`, `FixedTileStrategy.java` — clip flags and routes

## Deobfuscated client read for names (not copied)

Darkan's build-727 client in Kotlin, at
`C:/Users/User/Documents/Runescape/727-references/darkan-bot-dev/darkan-bot-dev`: the same
client with readable names. Read beside the reference Java for what things are called; the
Java decides behaviour where they differ.

- `com/jagex/game/runetek5/client/input/camera/Camera.kt` — `updateFollowPlayerCamera`
  (`Class363.method6286`), `cameraBaseX` / `cameraBaseY` (the focus), `cameraRandomisationH` /
  `cameraRandomisationV` (its offset)
- `com/jagex/game/runetek5/entity/pathingentity/player/PlayerUpdate.kt` — `decodeSkipcount`
  (`Class399.decodeSkip`)
- `com/jagex/game/runetek5/client/menu/MiniMenuBuilder.kt`, `MiniMenu.kt`
  (`raycastTerrainIntersection`), `scene/MapRegion.kt` (`calculateViewRange`) — picking the
  ground tile under the mouse and the view range (its fog multiplier is its own, not followed)
- `com/jagex/game/runetek5/scene/CollisionMap.kt` — `flagBoundary`, `flagBlockedObject`, `flagBlockedDecoration`, `setUnwalkable`: the clip flags' names
