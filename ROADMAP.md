# Roadmap

What has been ported so far (mostly by an AI assistant following `docs/PORTING.md`), what is
left, and where each piece lives. Rules and citations for everything below are in
`MECHANICS.md`. Keep this file current: move items to "Done" when they land, and write down
what you find is left.

Reference names below are classes/methods of the reference client (`src/main/java/com/jagex`).

## Done

### Cache and server
| Feature | Our code | Reference |
|---|---|---|
| Cache files: data file, indexes, containers (none/bzip2/gzip), reference tables, archives, XTEA, OpenRS2 keys | `src/cache/{store,container,reference_table,archive,xtea,keys,buffer,writer}.rs` | `Index`, `JS5…`, `ByteBuf` |
| JS5 cache server and client (727 handshake + key, requests, 512-byte blocks, checksum table) | `src/js5/`, `src/protocol/js5.rs`, `src/net.rs`, `src/client/updater.rs` | `LoginProt`, `JS5StandardRequester_Sub1` |
| 727 world login: RSA block, XTEA, ISAAC, reply | `src/login/`, `src/protocol/login.rs`, `src/client/login.rs` | `Login.java` |
| Server process, status window, trace log, settings | `src/server_app.rs`, `src/loader.rs`, `src/screen.rs`, `src/trace.rs`, `src/config.rs` | — |

### Loading
| Feature | Our code | Reference |
|---|---|---|
| Loading stages and texts, fallback bar, index-33 loading screens, crossfades | `src/client/{loading_stage,loading_screen}.rs`, `loading_screen/elements.rs`, `src/cache/loading_screens.rs` | `LoadingStage`, `Class158_Sub1_Sub2`, `GameTipsLoader`, `Class306` |
| Sprites, fonts and metrics, software raster | `src/cache/{sprite,font_metrics}.rs`, `src/client/{raster,font}.rs` | `SpriteDefinitions`, `FontMetrics`, `FontRenderer` |

### Login window (interface 744)
| Feature | Our code | Reference |
|---|---|---|
| Interface and CS2 script formats, opcode table | `src/cache/{interface,cs2,cs2_opcodes}.rs` | `IComponentDefinitions.readValues`, `CS2Script`, `CS2Instruction` |
| Entity defaults, enums, params, structs | `src/cache/{entity_defaults,enums,params}.rs` | `EntityDefaults`, `EnumDefinitions`, `ParamDefinitions`, `CacheableNode_Sub4` |
| CS2 machine and the instructions the login window reaches | `src/client/cs2/{mod,ops}.rs` | `CS2Executor`, `CS2Interpreter` |
| Live components, sub-interfaces, layout, 2D drawing | `src/client/interface/{mod,component,layout,render,assets}.rs` | `initSizes`, `method4204`, `method16099`, `method8044`, `IComponentDefinitions.render` |
| Mouse and keyboard, hook queues | `src/client/interface/input.rs`, `src/client/window.rs` | `ServerEnvironment.pulseSubInterfaces`, `client.method11768`, `CanvasKeyRecorder` |
| Logging in through the real form, connecting spinner and refusal messages | `cs2/ops.rs` (`instr6707`, `instr6709`), `client/mod.rs` | `Class155.method2635`, `Class455.method7558`, `Class9.anInt72` |
| Game preferences, Auto Setup, Options panel (presets, volume sliders, stereo), sound button | `src/client/preferences.rs`, `cs2/ops.rs` | `GamePreferences` + preference classes, `ServerEnvironment.method8313` |

### Lobby
| Feature | Our code | Reference |
|---|---|---|
| Lobby login (type 19) and the lobby window | `src/client/{login,lobby}.rs`, `src/login/` | `MaterialProp35.lobbyLogin`, `Login.java` |
| Lobby packets (`VARP_SMALL` / `VARP_LARGE`), world list and sorting | `src/protocol/world_list.rs`, `src/client/interface/worlds.rs`, `src/login/world_list.rs` | `PacketDecoder`, `Class359`, `Class52` |
| PLAY into the world, logging out to the login screen | `src/client/mod.rs`, `Ui::log_out` | `Class78.method1384`, `CursorIndexLoader.logout` |
| Resizable window and screen size modes | `src/client/window.rs` | `Class507.method8725`, `Class46.method935` |

### World
| Feature | Our code | Reference |
|---|---|---|
| `REGION` (position, map keys) | `src/login/world.rs`, `src/client/login.rs` | `MapRegion.loadMapSceneNormal`, `Class197.method3203` |
| Map terrain (`m{x}_{y}`), floors (underlays, overlays), colour table | `src/cache/{map_terrain,floors}.rs`, `src/client/palette.rs` | `Class329.decodeTileMasks`, `UnderlayDef`, `OverlayDef`, `CutsceneCameraMovement.method1365` |
| Ground of all four planes: blend, shapes, ground blending, OpenGL lighting | `src/client/{ground,tiles}.rs` | `Class329.method5837/5845/5846`, `OpenGLGround` |
| Game camera (pitch, lift, zoom) | `src/client/camera.rs` | `Isaac.processCamera`, `LoadingStage.method6683` |
| Locations (`l{x}_{y}`, XTEA), object definitions | `src/cache/{map_locations,object_def}.rs`, `src/client/locations.rs` | `Class329_Sub1.method12461`, `ObjectDefinition` |
| Object models: build, ground fitting, lighting; walls and decorations | `src/cache/mesh.rs`, `src/client/object_model.rs` | `RSMesh`, `ObjectDefinition.method7971/8010`, `MeshRasterizer_Sub1`, `Class329_Sub1.method12459/12464/12465` |
| Roofs hidden over the player, bridges | `src/client/{scene,world}.rs` | `QuickchatMessageLoader.method7215`, `JS5Manager.method5492`, `SceneObjectManager.method3424` |

### Music
| Feature | Our code | Reference |
|---|---|---|
| Login track id (track 0) and packed track → MIDI file | `src/cache/music.rs` | `Class89.method1496`, `Node_Sub7` |
| Instrument patches (index 15) | `src/cache/music_patch.rs` | `Node_Sub14`, `Class110` |
| Vorbis instrument samples (index 14) | `src/cache/vorbis.rs` | `MIDIInstrument`, `Class255`, `Class338`, `Class265`, `Class259` |
| Synthesized instrument samples (index 4) | `src/cache/sound_effect.rs` | `SoundEffect`, `Instrument`, `Envelope`, `Filter` |
| MIDI reader and the notes each program plays | `src/cache/midi.rs` | `Class80`, `Node_Sub7.method12165` |
| MIDI player and voice mixing (22,050 Hz) | `src/client/music/` | `Node_Sub15_Sub2`, `Node_Sub15_Sub3`, `Node_Sub15_Sub5(_Sub1/_Sub2)`, `Node_Sub13` |

### Tooling
| Tool | Where |
|---|---|
| `--render-interface`, `RENDER_INPUT`, `--dump-interface`, `--dump-script`, `--script-scope`, `--scripts-using`, `--export-music`, `--render-music`, `--map-terrain`, `--map-locations` | `src/tools.rs`, `src/main.rs` |
| `--self-test` (loading → login window → typing → login → lobby → logout → PLAY → world picture and roof check) | `src/selftest.rs` |
| Citations | `scripts/cite.py` |
| Porting method and assistant skills | `docs/PORTING.md`, `.claude/skills/` |

## Next

1. **Login music** (person's request). Left, in order:
   - done: MIDI reader timing, player and mixer (`--render-music 0` renders the login track);
     not yet: controller 81 retrigger and the voice budget;
   - done: the login music plays at the login screen and in the lobby, fades out on entering
     the world, and follows the login-music volume (`instr6629`) — `src/client/music/jukebox.rs`
     (`Class148` state machine) and `load.rs` (song + samples, rendered in a thread, played with
     macroquad audio). `DETAIL_MUSICVOL` only matters once in-game music exists.

   Map of the player (ported in `src/client/music/`; the reference's names are obfuscated, so
   these notes say what each part does):

   | Reference | What it does |
   |---|---|
   | `Node_Sub15_Sub2.method15131` | load a song: reset channels, read each track's first delta, find the earliest event (`anInt9717` track, `anInt9696` tick, `aLong9693` time) |
   | `method12230` (and its copies `method12240/41/42`) | render n output samples: advance song time by `division × 1,000,000 / 22050` per sample (`anInt9695`), stopping at each event time to run `method15127`, mixing voices in between (`Node_Sub15_Sub3.method12230`) |
   | `method15127` | run every event due now (`method15119`), loop the song when `aBool9716` and it ends |
   | `method15119` | MIDI messages: note off/on (`method15109`/`method15107`), controllers (bank 0/32, mod 1/33, portamento time 5/37, volume 7/39, pan 10/42, expression 11/43, sustain 64, portamento 65, (N)RPN 98–101, data entry 6/38 for pitch-bend range, all sound off 120, reset 121, all notes off 123, retrigger 81, 16/48 sample offset, 17/49 retrigger rate), program change (`method15189`), pitch bend (`method15091`) |
   | `method15107` | note on: portamento glide if enabled, else a new voice (`Node_Sub13`) from the program's patch (`Node_Sub14`): volume `v² × patch volume × patch base + 1024 >> 11`, pan, pitch, exclusive class cuts the previous voice |
   | `method15108` | voice volume: channel volume × expression, squared, × note volume, × player volume (`setVolume`), × `anInt9690`, × channel master (`method15095`), × decay (`0.5^(t × 1.953125e-5 × decay)`), × release then attack envelopes (`Class110`) |
   | `method15120` | voice rate: pitch + bend × range + vibrato (`sin(0.01227 × phase)`), `rate = sampleRate × 256 × 2^(pitch × 3.2552e-4) / 22050`, at least 1 |
   | `method15122` | pan: centre 8192, scaled by the note's pan |
   | `method15128` | per 10 ms (22050/100 samples) voice update: portamento decay, rate, envelopes, end of the voice when an envelope finishes |
   | `Node_Sub15_Sub3.method12230` | mix every voice in 10 ms steps, calling `method15128` between steps |
   | `Node_Sub15_Sub5` | a playing sample: position (8.8 fixed point, `anInt9853`), rate (`anInt9855`, negative = backwards), volume/pan with linear ramps over n samples (`method15321`, `method15322` fade out, `method15324` next ramp), pan law `left = v·√((16384 − pan)/8192)`, `right = v·√(pan/8192)` (`HostNameIdentifier.method488`, `MaterialPropTexture.method15390`), loops (`anInt9854`: −1 forever, n times, ping-pong when the sample's loop flag is set) |
   | `Node_Sub15_Sub5_Sub1.method12230` | float samples: walk loop segments forwards (`method15769`) / backwards (`method15732`); each segment uses linear interpolation `s·256 + (next − s)·frac`, × (volume × 128) `>> 6` into the int mix buffer, stereo when `Class253.aBool3115` |
   | `Node_Sub15_Sub5_Sub2` | the same for the 8-bit synthesized samples |
   | `Class253` | output device: 22050 Hz (`anInt3129`), stereo flag from the mono/stereo preference |
2. **Server side of login**: done — accounts with passwords (`src/login/accounts.rs`), replies
   3 (wrong password) and 5 (already logged in).
3. **Saving preferences**: done (`preferences.dat`, `src/client/preferences.rs`), with the
   reference's off-by-one read kept (see MECHANICS.md).
4. **Login window extras**: the message-box style (`SETUP_MESSAGEBOX`, `Class174`), key bindings
   (`aBool1424`). Done: dragging (volume sliders) and the scroll wheel (`src/client/interface/drag.rs`).
   No login component uses a click mask or a rotated sprite.
5. **3D scene behind the login window** (content type 1337, camera instructions `instr6650`,
   `instr6651`, `instr6675`, `instr6162`): needs models, the map and the 3D toolkit.
6. **Lobby** (wanted, though the reference has `Loader.LOBBY_ENABLED = false`; `server.cfg`
   `lobby = true`, the default). Plan, in order:
   1. Lobby login (type 19, `MaterialProp35.lobbyLogin`): the server answers with the lobby
      reply body (`Login.java:409-450`, `protocol::encode_lobby_ok`), the client reads it and
      opens the lobby window (entity defaults opcode 6, `QuickChatMessage.method6161`).
      **Done.**
   2. The lobby window draws and reacts: port the script instructions its scripts use (same
      way as the login window: survey its scripts, port what stops them). The lobby
      builds, fades in and shows its main panel (tabs, Player Info, Play). The tabs switch on
      click (option clicks, `IF_OP` packets). Next: Play (step 5).
   3. Lobby connection and packets: keep the lobby connection open after the reply; the
      client's packet reader (`PacketDecoder`, ISAAC opcodes, sizes from `ServerProt`) and
      what the server sends a lobby player (variables, friends and chat status, messages). **Started**: the connection stays open, packets are read
      by `ServerProt` size, `VARP_SMALL` / `VARP_LARGE` are applied, the server sends
      variable 2567.
   4. World list: the client asks for it, the server sends the list packet, the world
      selection screen shows and sorts it. **Done**: `REQUEST_WORLD_LIST` → `WORLD_LIST`
      (`src/protocol/world_list.rs`), client list and sort (`src/client/interface/worlds.rs`),
      server's list (`src/login/world_list.rs`: its one world, players online). Not yet: pinging
      worlds (the Ping column shows "-").
   5. Play: from the lobby to a world login on the chosen world (`Class78.method1384`,
      game state 7 → `worldLogin`), then `REGION` as now. **Done** (`src/client/mod.rs`, the
      lobby stage; server: lobby and world are separate places for "already online").
   6. Logging out: from the lobby to the login screen is **done** (`instr6623` →
      `CursorIndexLoader.logout`, `Ui::log_out`). From the world back to the lobby needs the
      game's own interface first.
7. **In the world** (see "World" above for what is done). Left, in order:
   1. Textures: **started** — the texture table and every material's operation graph are read
      (`src/cache/material.rs`, `--texture-stats`); pixels are generated row by row
      (`src/client/texture.rs`, `--render-texture`) with operations 0, 1, 2, 3, 5, 6, 7, 8, 9,
      10, 4, 11, 12, 13, 15, 17, 18, 19, 20, 21, 22, 23, 26, 27, 28, 29, 30, 32, 34, 35, 36, 38,
      39 (2302 of 2315 textures: sprites, textures inside textures, drawn shapes, bricks,
      blocks, stripes, waves, a kaleidoscope and scratches included). Next: the last
      operations — 14 (4), 37 (3), 16, 24, 25 — then textured faces on the ground (water) and
      on models (leaves, roof tiles).
   2. The game packets (`PacketDecoder`) and the game interface; walking (the player moves, the
      scene reloads at its edge).
   3. Smaller pieces: scenery shadows (`Ground.LA`), ground fitting mode 3 (`method11281`),
      animated objects, the camera's bridge lift, occlusion culling, the draw distance, the sky
      and fog (`Atmosphere`).
8. Remove the plain fallback form (`src/client/login_screen.rs`, `src/client/ui.rs`) once nothing
   needs it (it is used when the cache has no login window, and to start the login).

## Not part of the port

`src/game/`, `src/render/`, `src/input.rs`, `src/world.rs` and `src/mechanics.rs` are leftovers
of an early prototype. They are not compiled (not declared in `main.rs`) and are ignored by git;
they can be deleted.

## Where things are

| Path | What |
|---|---|
| `src/cache/` | cache formats, one file per format |
| `src/client/` | the client: updater, loading screens, interfaces, CS2, preferences, login |
| `src/client/interface/` | interfaces: components, layout, drawing, input, assets |
| `src/client/cs2/` | CS2 script machine (`mod.rs`) and instructions (`ops.rs`) |
| `src/protocol/` | the wire format both sides share: JS5, login packet and replies, RSA, ISAAC, packet tables |
| `src/net.rs` | server: the port, one thread per connection, picks the service |
| `src/js5/` | server: the cache service |
| `src/login/` | server: logins, accounts, what lobby and world players are sent first |
| `src/players.rs` | server: one world's players and their indices (up to 2047; `world_capacity`, 2000 by default) |
| `src/tools.rs` | developer tools (see `docs/PORTING.md`) |
| `MECHANICS.md` | every rule with its reference function and offset |
| `docs/PORTING.md`, `.claude/skills/` | how to port |
| `server.cfg.example` | settings; copy to `server.cfg` and point it at your build-727 cache |
