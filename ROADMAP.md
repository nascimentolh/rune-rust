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
| JS5 cache server and client (727 handshake + key, requests, 512-byte blocks, checksum table) | `src/js5/`, `src/net.rs`, `src/client/updater.rs` | `LoginProt`, `JS5StandardRequester_Sub1` |
| 727 world login: RSA block, XTEA, ISAAC, reply | `src/login/`, `src/client/login.rs` | `Login.java` |
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

### Music (in progress, see below)
| Feature | Our code | Reference |
|---|---|---|
| Login track id (track 0) and packed track → MIDI file | `src/cache/music.rs` | `Class89.method1496`, `Node_Sub7` |
| Instrument patches (index 15) | `src/cache/music_patch.rs` | `Node_Sub14`, `Class110` |
| Vorbis instrument samples (index 14) | `src/cache/vorbis.rs` | `MIDIInstrument`, `Class255`, `Class338`, `Class265`, `Class259` |
| Synthesized instrument samples (index 4) | `src/cache/sound_effect.rs` | `SoundEffect`, `Instrument`, `Envelope`, `Filter` |
| MIDI reader and the notes each program plays | `src/cache/midi.rs` | `Class80`, `Node_Sub7.method12165` |

### Tooling
| Tool | Where |
|---|---|
| `--render-interface`, `RENDER_INPUT`, `--dump-interface`, `--dump-script`, `--script-scope`, `--scripts-using`, `--export-music` | `src/tools.rs`, `src/main.rs` |
| `--self-test` (loading → login window → typing → login) | `src/selftest.rs` |
| Citations | `scripts/cite.py` |
| Porting method and assistant skills | `docs/PORTING.md`, `.claude/skills/` |

## Next

1. **Login music** (person's request). Left, in order:
   - playback timing of the MIDI reader (`Class80.method1403`, `method1406`);
   - MIDI player `Node_Sub15_Sub2` (`method15182` load, `method15098` play, `setVolume`) and voice
     mixing `Node_Sub15_Sub5_Sub1`; output at 22,050 Hz (`MaterialProp33.method15418`);
   - start/stop at the login screen and on `instr6629` / `DETAIL_MUSICVOL`
     (`GameState.setGameState`, `Class339.method6047`, `ProcessorSpecs.method7730`,
     `MaterialProp33.method15420`); play through macroquad audio.

   Map of the player, already read line by line (port next; the reference's numbers are
   obfuscated, so these notes say what each part does):

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
2. **Server side of login** (not everything is client-side): accounts and password checks so the
   server sends the reference's refusal codes (`src/login/mod.rs` refuses only empty names or
   passwords today).
3. **Saving preferences** to a file like the reference (`Class190.savePreferences`,
   `FixedTileStrategy.method12791`); today they last one run.
4. **Login window extras**: the message-box style (`SETUP_MESSAGEBOX`, `Class174`), key bindings
   (`aBool1424`), sprite click masks, dragging, the scroll wheel, rotated sprites (`angle2d`).
5. **3D scene behind the login window** (content type 1337, camera instructions `instr6650`,
   `instr6651`, `instr6675`, `instr6162`): needs models, the map and the 3D toolkit.
6. **After login**: the packets the reference reads after the login reply (`Login.java` from
   about line 487, `PacketDecoder`), then the lobby or game world.
7. Remove the plain fallback form (`src/client/login_screen.rs`, `src/client/ui.rs`) once nothing
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
| `src/js5/`, `src/login/`, `src/net.rs` | server services |
| `src/tools.rs` | developer tools (see `docs/PORTING.md`) |
| `MECHANICS.md` | every rule with its reference function and offset |
| `docs/PORTING.md`, `.claude/skills/` | how to port |
| `server.cfg.example` | settings; copy to `server.cfg` and point it at your build-727 cache |
