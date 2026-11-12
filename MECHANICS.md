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

## Login screen — in progress

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
| Tools: `--dump-interface`, `--dump-script`, `--script-scope` | — | `tools.rs` |

Next: the interpreter (`CS2Interpreter` for those 171 opcodes), the interface layout (`aspect*Type`
modes) and renderer (`IComponentDefinitions.render`), then the login form's input handling.
The placeholder login screen (`client/login_screen.rs`) stays until then.

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
