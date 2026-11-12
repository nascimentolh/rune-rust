# Game client + server — format, protocol and rules sheet

One program, always two processes: opening it starts the **client** (loading screen →
login screen); the client starts the **server** (`--server`) as its own process when none
is running. Code standard: unused/dead code fails the build, `cargo clippy --all-targets`
must be clean, `cargo fmt` style (`rustfmt.toml`).

## Where the reference values come from

The original (the open727Client project) has **no
compiled game program**: the game is Java source (`src/main/java/com/Loader.java` plus 1,054
files in `com/jagex/`). Its only native binaries are third-party Discord Rich Presence and JNA
libraries inside `lib/discord-rpc.jar` (evidence `ev_b344e819…`, finding `fnd_1`). So there are no
function addresses to cite: every value below cites the **Java file and line** it was read from,
read line by line in Deep mode. Paths are relative to `src/main/java/com/`.

## Recorded evidence (Craft Anything, Deep mode)

`core_target_open` opens the original as a folder, and `native_strings_find` /
`native_function_pseudocode` refuse it: "A folder is not native machine code" (finding `fnd_1`).
In place of function addresses, the key values were located at byte offsets in the original's files
with `native_bytes_find` (findings `fnd_2`, `fnd_3`):

| Value used by this build | Original file @ offset | Evidence |
|---|---|---|
| Port 5555 | `Loader.java` @ 0x201 | `ev_5952c050…` |
| Build 727 | `Loader.java` @ 0x177 | `ev_2fd51173…` |
| "Checking for updates - " | `jagex/LocalizedText.java` @ 0xacb | `ev_d2ec1641…` |
| Fallback box 303 × 33 | `jagex/Class158_Sub1_Sub2.java` @ 0x7bd | `ev_c2125efd…` |
| JS5 service opcode 15 | `jagex/LoginProt.java` @ 0x4f | `ev_193a5e58…` |
| Login build 727 written by the client | `jagex/Login.java` @ 0x2615 | `ev_24d0e7d0…` |
| Stage 6 runs 5 → 98 % | `jagex/LoadingStage.java` @ 0x59b | `ev_87c13bdc…` |
| JS5 data in 512-byte blocks | `jagex/JS5StandardRequester_Sub1.java` @ 0x89fb | `ev_aa6f1201…` |
| Login window id read as u24 (opcode 5 → 744) | `jagex/EntityDefaults.java` @ 0x94c | `ev_3a0e5251…` |
| Script opcodes looked up in the client's own table (needs a build-727 cache) | `jagex/CS2Script.java` @ 0x3b4 | `ev_c8fb15bc…` |
| `CALL_CS2` = opcode 902 with an int operand | `jagex/CS2Instruction.java` @ 0x2ce | `ev_30f55b63…` |
| Component text "RuneScape" → "Open 727" | `jagex/IComponentDefinitions.java` @ 0x321c | `ev_4acdeabd…` |
| On-load scripts run with a 2,000,000-instruction limit | `jagex/Connection.java` @ 0x23d | `ev_8c194f6f…` |
| Script call stack holds 50 frames | `jagex/CS2Executor.java` @ 0x4bb | `ev_bb414e68…` |
| Entity defaults are defaults file 3 (index 28) | `jagex/DefaultsFile.java` @ 0xca | `ev_37e03295…` |
| Volume preferences default to 127 | `jagex/SoundLevelPreference.java` @ 0x2ad | `ev_fb67d4c8…` |
| `PROFILE_CPU` times 10,000 triangle draws | `jagex/Class241.java` @ 0x1b0 | `ev_88989503…` |
| Fixed-size window reports mode 1 (`instr6644`) | `jagex/Class158.java` @ 0xb1 | `ev_c0267b01…` |
| Key code table: Backspace → 85, Tab → 80, Enter → 84 (`KEY_CODES`) | `jagex/CanvasKeyRecorder.java` @ 0x14e | `ev_69141056…` |
| Login names up to 320 characters (`instr6707`) | `jagex/Class155.java` @ 0x16b | `ev_454f344d…` |
| Input hooks run with a 200,000-instruction limit | `jagex/CS2Executor.java` @ 0x161f | `ev_2fdfd496…` |
| Click queues the press hook `anObjectArray1386` | `jagex/client.java` @ 0xb71b | `ev_9eadbcd0…` |
| Printable characters: 32–126, 160–255, 8364, 338, 8212, 339, 376 | `jagex/Node_Sub25.java` @ 0x50c | `ev_6cf80fbc…` |
| Size mode 2: base × parent >> 14 (`layout.rs` `size`) | `jagex/Class484.java` @ 0x21d | `ev_3b3e49cd…` |
| Position mode 1: base + (parent − width) / 2 (`layout.rs` `position`) | `jagex/Class246.java` @ 0x368 | `ev_ca2558ce…` |
| Transparent figure alpha 255 − (t & 0xff) (`render.rs` `with_alpha`) | `jagex/IComponentDefinitions.java` @ 0xbcdb | `ev_3317dd77…` |
| Every-tick hook queue drained first (`input.rs` `tick`) | `jagex/client.java` @ 0x181a0 | `ev_540a8740…` |
| Login reply code starts at -2 (`Ui::login_reply`) | `jagex/Class9.java` @ 0x2f | `ev_2668b1a7…` |
| Starting a login sets -3 (`instr6707`) | `jagex/Class455.java` @ 0x1b3c | `ev_1c4b3d1e…` |
| Empty name or password sets 3 (`instr6707`) | `jagex/Class455.java` @ 0x1be1 | `ev_9ee4030d…` |
| Failed connection sets -4 (`login::CONNECTION_FAILED`) | `jagex/Login.java` @ 0x8f50 | `ev_7823948a…` |
| Scripts read the code with `instr6709` | `jagex/CS2Interpreter.java` @ 0x2514f | `ev_0155a3ed…` |
| Auto Setup: CPU profile ≤ 104 ms → level 4 (`auto_setup`) | `jagex/ServerEnvironment.java` @ 0x462 | `ev_631dc73e…` |
| Auto Setup: ≤ 1002 ms → level 2 | `jagex/ServerEnvironment.java` @ 0x532 | `ev_25b49c27…` |
| Level 4 preset sets water 2 (`preset`) | `jagex/EquipmentDefaults.java` @ 0x52a | `ev_51257ce2…` |
| Level 1 preset sets graphics 1 | `jagex/MaterialProp14.java` @ 0x6ca | `ev_a6f396b5…` |
| Skyboxes refused without toolkit effects (`set`) | `jagex/Preference_Sub14.java` @ 0x25c | `ev_628d124f…` |
| Login music id from defaults file 4 (`music::login_track`) | `jagex/IndexLoaders.java` @ 0x2c06 | `ev_bad097fc…` |
| First opcode-1 track wins | `jagex/Class89.java` @ 0x242 | `ev_93407b21…` |
| Rebuilt track starts with `MThd` (1297377380) (`music::to_midi`) | `jagex/Node_Sub7.java` @ 0x1491 | `ev_269e43b9…` |
| Tempo events written as meta 0x51 | `jagex/Node_Sub7.java` @ 0x1af1 | `ev_02bcc95d…` |
| Music patches are index 15 (`music_patch`) | `jagex/IndexLoaders.java` @ 0x22fa | `ev_571378db…`|
| Loop bit: ((sample − 1) & 2) << 14 | `jagex/Node_Sub14.java` @ 0x107a | `ev_25887886…` |
| Even sample kind → synthesized sound, odd → Vorbis instrument | `jagex/Node_Sub14.java` @ 0x383a | `ev_7592c476…` |
| Curve division rounds toward −∞ (`div_floor`) | `jagex/Class371.java` @ 0x407 | `ev_f39b34e9…` |
| Note pan = (stored byte + 16) << 2 | `jagex/Node_Sub14.java` @ 0x14a8 | `ev_b2938b41…` |
| Patch volume = stored byte + 1 (`base_volume`) | `jagex/Node_Sub14.java` @ 0x19b8 | `ev_16f473d1…` |
| Vorbis setup is file 0/0 of index 14 (`vorbis::Setup`) | `jagex/MIDIInstrument.java` @ 0x15b1 | `ev_fb4a8802…` |
| Packed float = mantissa × 2^(exponent − 788) (`unpack_float`) | `jagex/MIDIInstrument.java` @ 0x1c8c | `ev_7bbd5b50…` |
| Negative loop end is bit-inverted and sets the loop flag | `jagex/MIDIInstrument.java` @ 0x1e17 | `ev_dbed5b87…` |
| Floor-1 ranges 256, 128, 86, 64 (`FLOOR_RANGES`) | `jagex/Class338.java` @ 0x31 | `ev_0bc4b20a…` |
| `getFile(id)` = file `id` of archive 0 when the index has one archive | `jagex/Index.java` @ 0x1c73 | `ev_ee17bd64…` |
| Enums split 8 bits: archive `id >>> 8`, file `id & 0xff` (`Assets::enumeration`) | `jagex/SharedConfigsType.java` @ 0xb0 | `ev_d087fbb4…` |
| Synth noise from `new Random(0L)` (`sound_effect::noise`) | `jagex/Instrument.java` @ 0x15c | `ev_638f09dc…` |
| Synth sine = sin(i / 5215.1903) × 16384 | `jagex/Instrument.java` @ 0x239 | `ev_5fab21f5…` |
| Oscillator pitch × 1.0057929410678534^pitch | `jagex/Instrument.java` @ 0xacf | `ev_45a1db29…` |
| Sounds mix at 22,050 Hz: samples = ms × 22050 / 1000 | `jagex/SoundEffect.java` @ 0x891 | `ev_c2f613b8…` |

## Client start-up and connection — read from the reference

| Value | Reference | In this build |
|---|---|---|
| Build 727, minor 1 | `Loader.java:17-18` `MAJOR_BUILD = 727`, `MINOR_BUILD = 1` | `config.rs` defaults, `server.cfg` |
| Port 5555 (lobby, world and JS5 world 177) | `Loader.java:20-21` `WORLD_PORT = LOBBY_PORT = 5555`, `getPort` | `config.rs` defaults, `server.cfg` |
| RSA public exponent 65537 | `Loader.java:26` | `login/rsa.rs` `PUBLIC_EXPONENT` |
| Lobby off → always world login (type 16) | `Loader.java:28`, `jagex/MaterialProp35.java:14` `lobbyLogin` → `ParticleArchive1Def.worldLogin` | Rust client sends type 16 |
| ISAAC **on** (the `USING_ISAAC = false` constant at `Loader.java:27` is never read anywhere) | `jagex/Login.java:269-275` | server reads with the seeds, writes with seeds + 50 |
| JS5 key = client parameter 17 `ev9+VAp5/tMKeNR/7MOuH6lKWS+rGkHK` | `Loader.java` params, `jagex/client.java:1804-1805` | `config.rs` `js5_key`, sent by the Rust client, logged by the server |
| Window 765 × 503 | `jagex/Class304.java` (`765` in the base width), fallback bar maths | `client/ui.rs` `WIDTH`, `HEIGHT` |

## Loading screens — read from the reference and ported

| Value | Reference | In this build |
|---|---|---|
| 18 stages, their texts, min/max %, which interpolate | `jagex/LoadingStage.java:25-42` | `client/loading_stage.rs` `STAGES` |
| Text = stage text + % ; on leaving a stage show its max | `jagex/Preference_Sub20.java` `method12808` | `Status::update` |
| "Checking for updates - ", "Fetching Updates - ", "Loading - please wait." | `jagex/LocalizedText.java:18,22-23` | `loading_stage.rs`, `loading_screen.rs` |
| Fallback box 303 × 33 at centre −152, −18; 3 px per percent; inner 301 × 31 black | `jagex/Class158_Sub1_Sub2.java:52-58` `method15535` | `loading_screen.rs` `draw_fallback_bar` |
| Fallback colours 0x8C1111 fill/outline, white text | `jagex/client.java:47-49` `aColorArray7186/7199/7200[0]` | `FALLBACK_FILL`, `FALLBACK_OUTLINE` |
| Fallback font Helvetica bold 13 (Arial on Windows) | `jagex/Class158_Sub1_Sub2.java:27` | `FALLBACK_FONT_FILE`, size 13 |
| Fallback used because the applet's bar images aren't shipped | `jagex/Class304.java` `method5399` (resources missing → `method5398`) | fallback until index 33 arrives |
| Loading thread frame 20 ms (bar scroll clock) | `jagex/Class306.java:134` | `LOADING_FRAME_MS = 20` |
| Indexes for loading screens: 33 (defs), 32 (images; 34 only if images can't be decoded), 13 (font metrics) | `jagex/IndexLoaders.java:104-107` | `client/updater.rs` stage 1 |
| Index-33 header: version ≥ 4, 10 element tags 1,2,2,2,1,1,1,2,1,2 | `jagex/GameTipsLoader.java:19`, `jagex/Class60.java:7-16`, `jagex/Class112.java` `method1870` | `cache/loading_screens.rs` `TYPE_TAGS` |
| Default group 0 | `jagex/Preference_Sub11.java:50` | `DEFAULT_GROUP` |
| Switching: next screen when the current one's time has passed; wrap after the last | `jagex/Preference_Sub20.java:80-90`, `jagex/Class299.java` `method189` | `LoadingScreens::advance` |
| Crossfade: old at 255−a, new at a, a = 255·t / old fade | `jagex/Class306.java` `run` | `LoadingScreens::draw` |
| Element decoders (10 types) | `Class393.java` `method6745` and the decoders it calls | `decode_element` |
| Image element placement (anchor + offset) | `jagex/Class42.java` `method20`, `Class356/353` anchors | `elements.rs` |
| Text element = paragraph (wrap, h/v align, line height, colour, shadow) | `jagex/Class51.java` `method20`, `FontRenderer.java:382-447` | `client/font.rs` `draw_paragraph` |
| No wrapping when the box is shorter than two lines | `jagex/FontRenderer.java:394` | `draw_paragraph` |
| Image bar: caps, edges, fill/empty tiling; scroll = speed × frames / 10 mod width | `jagex/Class52_Sub2.java` `method1081/1077`, `Class52_Sub2_Sub1.java:16` | `draw_image_bar` |
| Bar text centred at h/2 + offset + 4; " (stage)" after 10000 ms without change | `jagex/Class52.java:84` `method20` | `BAR_TEXT_DROP`, `STALL_MS` |
| Bar smoothing between percentages | `jagex/Class52.java` `method1078` | `BarClock::fill` |
| Sprites: palette (stored 0 → 1), index 0 transparent, row/column order, alpha plane | `jagex/SpriteDefinitions.java:67` `decode` | `cache/sprite.rs` |
| Fonts: widths, kerning from edge profiles, spacing = start + count of glyph 32 | `jagex/FontMetrics.java:74`, `jagex/Utils.java` `calculateKerning` | `cache/font_metrics.rs` |
| Glyph tinting (solid if opaque colour, else blended), shadow at +1,+1 only if its alpha ≠ 0 | `jagex/FontRenderer_Sub1.java` `UA`, `FontRenderer.java:250-253` | `Raster::draw_glyph`, `Font::draw_line` |
| Images decoded as ordinary files; 4-channel JPEG = YCbCr + alpha (Java's rule) | `jagex/Class103_Sub1.java` `method14490` | `elements.rs` `decode_ycca_jpeg` |

Verified against the person's cache: all 31 screens of group 0 decode exactly and draw
(self-test step `loading-screens`, picture `shots/loading-screen.png`).

## Login screen — drawn and usable

**The cache must be build 727.** Each build renumbers its CS2 opcodes. The client reads them with
its own table (`jagex/CS2Script.java` `getOpcode` → `CS2Instruction.getByOpcode`), so the scripts in
a build-718 cache come out as nonsense (script 1172 opens with `IF_SETONMOUSEOVER` carrying a 4-byte
operand it doesn't have), and the login screen can't be built, in the Java client or here. With
OpenRS2 cache #309 (build 727, 2012-07-24) every script decodes. Interfaces, sprites, fonts and
loading screens keep the same format in both builds, which is why they worked on the 718 cache.

What the reference does (game state 5):
- `ENTITY_DEFAULTS.loginWindow` = **744** (index 28 file 3, opcode 5; `jagex/EntityDefaults.java:66`),
  set by `jagex/Class348.java` `method6175`.
- Interface 744 has 135 components. It's a shell: frames, a hidden display dialog, the world button,
  a central 512 × 334 slot (#116) and background tiles with no picture yet (sprite −1).
- Its root (#17) runs on-load script **1172**, which calls script 25 and then 2708 with components
  of interface 882. Everything visible is built by scripts: **62 scripts, 171 opcodes**
  (`--script-scope 1172`).

Ported so far:
| Part | Reference | Here |
|---|---|---|
| Interface component format (all kinds, options, 21 hooks, 5 int lists, the "Open 727" text tweak) | `IComponentDefinitions.readValues`, `decodeScript`, `decodeScript3` | `cache/interface.rs` |
| CS2 script format (trailer, switch tables, operands) | `CS2Script` | `cache/cs2.rs` |
| Opcode table (1,006 entries, generated from the enum) | `CS2Instruction.java` | `cache/cs2_opcodes.rs` |
| Entity defaults (login window), enums, params, structs | `EntityDefaults`, `EnumDefinitions`, `ParamDefinitions`, `CacheableNode_Sub4` | `cache/entity_defaults.rs`, `cache/enums.rs`, `cache/params.rs` |
| Script machine: stacks, locals, `CALL_CS2`/`RETURN` frames (50 deep), instruction limit, hook arguments with the magic source id/slot values, `aBool7022` second active component | `CS2Executor.executeScript`, `executeHookInner` | `client/cs2/mod.rs` |
| Every instruction the login window's scripts reach (arithmetic, strings, varc, enums, structs, `IF_*`, `CC_*`, hook setters via `popParamsDynamic`, sub-interfaces, preferences, client state) | `CS2Interpreter` and its helpers | `client/cs2/ops.rs` |
| Live components, created children (`slotChildren`), sub-interfaces (`OPEN_INTERFACES`), varc | `IComponentDefinitions`, `IdentiKitIndexLoader.openSub`, `Class351.closeChildren` | `client/interface/` |
| Layout: size modes 0/1/2/4, position modes 0–5, scroll clamping, containers, created children, sub-interfaces | `initSizes`, `method4204`, `method16099`, `method8044`, `method7403`, `method913` | `client/interface/layout.rs` |
| Drawing: containers with clip and scroll, figures (filled/outline, transparency), text (font, alignment, shadow, line height, max lines), sprites (flips, border, shadow, tint, stretch, tiling), lines | `IComponentDefinitions.render`, `method2048`, `SpriteDefinitions.method1524/1525/1526/1536/1554` | `client/interface/render.rs`, `assets.rs` |
| Game preferences as a fresh install has them, on the software toolkit | `GamePreferences`, each `Preference` subclass | `client/preferences.rs` |
| Opening the login window: layout, then every component's on-load script in order (limit 2,000,000) | `Class348.method6175`, `Connection.runIComponentScripts` | `client/interface/mod.rs` |
| Input once per 20 ms tick: press, held, release, hover, leave, every-tick and key hooks, then the three hook queues in order (deleted created components skip theirs) | `ServerEnvironment.pulseSubInterfaces`, `client.method11768`, `client.java:2788-2890` | `client/interface/input.rs` |
| Hook arguments for mouse x/y (relative to the component) and key code/char; target component -1 | `CS2Executor.executeHookInner` | `client/cs2/mod.rs` |
| Key codes: Java `VK_*` → client code table, codes with bit 0x80 dropped on press, presses (char 65535) then typed characters (code -1), typed-character filter | `CanvasKeyRecorder.anIntArray8044`, `method13378`, `method3235`, `Class175.method2962` | `client/window.rs` |
| Log in from the window: `instr6707` gives name and password (≤ 320 characters) | `Class155.method2635` | `client/cs2/ops.rs` |
| Auto Setup with no hardware toolkit: CPU profile ≤ 104 ms → level 4, ≤ 504 → 3, ≤ 1002 → 2, else 1; the level's preset; the software toolkit stays; `instr6852` pushes toolkit then level | `ApplyHitmarkCutsceneAction.method14641`, `ServerEnvironment.method8313` | `client/preferences.rs` `auto_setup` |
| Detail presets 4/3/2/1 (also MIN/LOW/MID/HIGH), set in the reference's order through `setValue` (`checkValid` 3 refuses: skyboxes, bloom and anti-aliasing on the software toolkit, scenery shadows without textures) | `EquipmentDefaults.method11248`, `Node_Sub40.method13300`, `Class119.method2077`, `MaterialProp14.method15393` | `preferences.rs` `preset`, `set` |
| Saved toolkit (3 refused, no DirectX) and graphics reset (`instr6922`) | `CS2Interpreter.method6729` | `client/cs2/ops.rs` |
| Volume sliders (music, sound effects, area sounds, voice over, login music `instr6629`), stereo/mono | `method2554`, `method3599`, `method12718`, `method3200`, `method4899`, `method3942` | `client/cs2/ops.rs` |
| Login reply code the window's scripts read (`instr6709`, script 2948): -2 none, -3 at once when the login starts (3 if name or password is empty), then the server's reply code, 2 when logged in, -4 when the connection fails | `Class9.anInt72`, `Class455.method7558`, `FontRenderer_Sub1.method13784`, `Login.java:108/315/568` | `Ui::login_reply`, `client/login.rs` |
| Tools: `--dump-interface`, `--dump-script`, `--script-scope`, `--render-interface` (with `RENDER_INPUT` clicks and typing) | — | `tools.rs` |

Every on-load script of interface 744 now runs to its end. On a fresh install the scripts open the
"Available graphics options have changed" dialog over the background art, as the reference does.

Clicking "Leave Alone" opens the login box; clicking a box and typing fills it (the password shows
as stars), and "Log In" makes the window's script call `instr6707`, which starts the 727 login.
While it connects the window's script 2946 (set by the Log In script 2945 as an every-tick hook)
shows "Logging In - Please Wait" with the spinner (2951); a refusal shows the reference's message
for its code, for example "Invalid login or password." with "Try Again". The self-test logs in
exactly this way.

Auto Setup, the Options panel (MIN/LOW/MID/HIGH, re-run Auto Setup, volume sliders, stereo/mono)
and the sound button work through the window's scripts; the values are kept for this run (the
reference also saves them to a preferences file, not done here).

Music — in progress. The login music is track **0** of index 6 (defaults file 4, opcode 1:
`IndexLoaders` stage → `Class89.method1496`). `cache/music.rs` rebuilds it as a standard MIDI
file exactly as `Node_Sub7`'s constructor does (15 tracks, division 960, 76,819 bytes; like the
reference, an end-of-track right after a tempo event is written without its `FF`, which the
client's own reader expects). `--export-music <id>` saves it to `shots/`.
Patches (index 15, one per MIDI program) decode as `Node_Sub14` does (`cache/music_patch.rs`): per
note the sample, pitch (loop bit 0x8000), volume and pan (with their curves), exclusive class and
envelope (`Class110`). All 247 patches of cache 727 decode; their notes use 1,187 Vorbis samples
(index 14, `MIDIInstrument`, odd sample kinds) and 6 synthesized sounds (index 4, `SoundEffect`,
even kinds), as `Node_Sub14.method12216` / `Class250.method4287/4290` pick them. Instrument samples
decode with the client's own Vorbis variant (`cache/vorbis.rs`): the shared setup is index 14
file 0/0 (`MIDIInstrument.method12261`: block sizes, codebooks `Class255`, floor-1 `Class338` with
its inverse-dB table, residues `Class265`, mappings `Class259`, modes), each sample is a header
(rate, length, loop start/end, negative end = flag) plus packets (`method12265`), decoded with
the reference's inverse MDCT and windows (`method12267`) until its length (`method12273`). All
1,187 samples the patches use decode (sample 1: 22,050 Hz, 62,110 samples, loop 42,292..62,110).
The reference decodes them a little at a time in the background (`Class2.method263`); here a
sample is decoded whole. `--export-music` saves one as WAV. Synthesized
samples (index 4) are made as `SoundEffect`/`Instrument`/`Envelope`/`Filter` make them
(`cache/sound_effect.rs`): up to ten instruments of up to five oscillators (square, sine, saw,
noise from `java.util.Random(0)`), pitch/volume modulation, gating, echo and the moving
pole/zero filter, mixed to signed 8-bit at 22,050 Hz with loop points in ms × 22050 / 1000. All
6 the patches use synthesize (e.g. sound 3711: 110,250 samples). The MIDI file is read as `Class80` does (`cache/midi.rs`: track cursors, running status kept
for meta events too, tempo changes moving the time base), and `Node_Sub7.method12165` lists the
notes each program plays (bank select 0/32, program change, note-on with velocity > 0; channel 9
starts in bank 128): the login track plays 11 programs (14, 15, 47, 49, 60, 168, 259, 277, 278,
279, 283), 144 program notes, so only their samples need loading. Still to port before it plays:
the MIDI player (`Node_Sub15_Sub2`) and mixer, then starting it
(`Class339.method6047`, state machine in `ProcessorSpecs.method7730` / `MaterialProp33.method15420`).
Until then the volume settings are stored but nothing plays. Also not yet: the 3D scene behind the window (content type 1337, camera instructions are popped and
ignored), models, rotated sprites, the message box style (`SETUP_MESSAGEBOX` is checked and dropped;
login messages don't need it), retrying a failed connection 3 times (`anInt104`), key bindings (`aBool1424`), the sprite click mask, dragging, the
scroll wheel, and hooks waiting on var/inventory/stat changes (none exist before login).

## Cache on disk — verified on the real cache
`main_file_cache.dat2` + `.idx0..36` + `.idx255`; 6-byte index entries; 520-byte sectors with
8-byte header (10 bytes above archive 65535); containers type 0/1/2; reference tables protocol 5–7.

## Map keys — verified
OpenRS2 `keys.json` for cache 302 in `data/keys.json`; XTEA, 32 rounds, encrypts everything after
the 5-byte container header. All 1987 keys decrypt their archives.

## JS5 (cache) protocol — read from the reference
| Item | Value | Reference |
|---|---|---|
| Handshake | 15 · size (9 + key length) · 727 i32 · 1 i32 · key string | `jagex/client.java:2310-2320` |
| Reply | status u8: 0 ok, 6 out of date, 7/9 full, 48 session expired; then 27 × i32 | `client.java:2263-2350`, `Class446.method7436` |
| After connecting the client sends | 6 · 3 (u24) · 0 (u16): "connected"; 2/3: logged in/out; 4 · key · 0: XOR key | `JS5StandardRequester_Sub1.java:552-559, 597-603` |
| Request | 6 bytes: 1 (urgent) or 0 · index u8 · archive i32 | `:711-725` |
| Response | index u8 · archive i32 · compression u8 (bit 0x80 = not urgent) · length i32 · data | `:765-791` |
| Block marker | 0xFF before every further 512 bytes (the 10-byte header counts toward the first 512) | `:798-832` |
| Checksum table (255/255) layout | still from 718-era knowledge — the client's parser isn't read yet | — |

## Login protocol — read from the reference
| Item | Value | Reference |
|---|---|---|
| Open | client sends 14 alone; server answers 0 | `jagex/Login.java:93, 100-117` |
| Packet | 16 · size u16 · 727 · 1 · reconnecting u8 · RSA block (u16 length + bytes) · XTEA block | `Login.java:165-266` |
| RSA plaintext | 10 · 4 random seeds · 0 i64 · password · i64 · i64, encrypted with `Loader` modulus, exponent 65537 | `Class94.method1587`, `ChatLine.getLoginMod`, `ByteBuf.applyRSA` |
| XTEA block | key = seeds; whole 8-byte blocks only; name flag (1 = text) · username · window mode · width · height · … | `Login.java:191-201`, `ByteBuf.encryptWithXtea` |
| Reply | 2 = ok (then size u8 + body); 21 wait; 1/23 retry; 29, 42, 45 extra bytes; other = error code | `Login.java:279-329` |
| OK body | rights · mod level · quick chat · verified email · flag · quick-chat-only · player index u16 · member · DOB u24 · members world · name | `Login.java:377-403` |
| Accounts | any name/password accepted (server choice; the reference is a client) | — |

**RSA key:** the reference encrypts the password with the public modulus in `Loader.java:25`. The
server needs the matching private exponent to read it, and that isn't in the client. Either put that
private key in `server.cfg` (`login_rsa_exponent` / `login_rsa_modulus`), or run `--gen-rsa` and put
the new modulus into `Loader.java`.

**Not ported yet:** the login packet's later fields (preferences, machine info, cache CRCs, world id),
which the Rust client doesn't send yet, and everything the server must send after login (`Login.java:487+`).
