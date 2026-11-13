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
| MIDI tempo starts at 500,000 µs per quarter (`midi::Reader`) | `jagex/Class80.java` @ 0x944 | `ev_aa47aaa5…` |
| Channel 9 starts on program 128 (drums) (`used_notes`) | `jagex/Node_Sub7.java` @ 0x31f5 | `ev_0a0070f9…` |
| Bank select: (value << 14) + (bank & −2080769) | `jagex/Node_Sub7.java` @ 0x3746 | `ev_77e0d66e…` |
| left pan gain v·√((16384 − pan)·1.220703125e-4) + 0.5 | `jagex/HostNameIdentifier.java` @ 0x910 | `ev_4f3c477f…` |
| voices update every 22050 / 100 samples (10 ms) | `jagex/Node_Sub15_Sub2.java` @ 0x6bb3 | `ev_4e9392e7…` |
| Voice decay = 0.5^(t × 1.953125e-5 × decay) (`music::Player::volume_of`) | `jagex/Node_Sub15_Sub2.java` @ 0x25f5 | `ev_46c22159…` |
| Voice rate = sample rate × 256 × 2^(pitch × 3.255208333333333e-4) / 22050 (`rate_of`) | `jagex/Node_Sub15_Sub2.java` @ 0x5758 | `ev_a34281c8…` |
| Song time scale 1,000,000 (division × 1e6 / 22050 per sample) (`Player::render`) | `jagex/Node_Sub15_Sub2.java` @ 0xb4 | `ev_4bc66bdb…` |
| Drag depth = settings >> 18 & 7 (`drag::depth`) | `jagex/IFTargetParams.java` @ 0x2f1 | `ev_72d22e0a…` |
| A drag counts as moved only after more than `anInt1381` ticks (`Ui::drag_tick`) | `jagex/EntitySpotAnim.java` @ 0xc5a | `ev_c144c615…` |
| Wheel hook's mouse y = wheel notches `anInt7191` (`input.rs`) | `jagex/client.java` @ 0x9413 | `ev_49bd3336…` |
| A later login try may take 2000 ticks (the first 250) (`login::LATER_TRY`) | `jagex/Login.java` @ 0x2c2 | `ev_71288bc4…` |
| Reply 23 tries again while fewer than 3 retries (`login::attempt`) | `jagex/Login.java` @ 0x43cb | `ev_36b01993…` |
| Too slow after the retries gives -5 (`login::TIMED_OUT`) | `jagex/Login.java` @ 0x4e2 | `ev_11754907…` |
| Preferences file version byte 27 (`preferences::VERSION`) | `jagex/GamePreferences.java` @ 0x404e | `ev_fbb95035…` |
| Preferences file name `preferences.dat` (`main.rs`) | `jagex/ParticleArchive1Def.java` @ 0x371 | `ev_339413b1…` |
| Volume check `< 0 && > 127` never resets (`Preferences::check`) | `jagex/SoundLevelPreference.java` @ 0x15e | `ev_73cece04…` |
| Custom cursors byte written but not read back (`Preferences::decode`) | `jagex/GamePreferences.java` @ 0x4735 / 0x16b7 | `ev_66b40571…`, `ev_4dc7e9a5…` |
| GPU score = meshes drawn × 1000 / elapsed ms (`gpu::profile`) | `jagex/Class383.java` @ 0x117d | `ev_63d0c8a3…` |
| OpenGL score × 1.1 before choosing (`Preferences::auto_setup`) | `jagex/ApplyHitmarkCutsceneAction.java` @ 0xcec | `ev_9db055dc…` |
| Score > 20077 → detail level 4 (then 10021 → 3, 5002 → 2) | `jagex/FontRenderer_Sub1.java` @ 0x424 | `ev_09e0fefd…` |
| CPU-only Auto Setup saves toolkit 0 (`software_setup`) | `jagex/ServerEnvironment.java` @ 0x69f | `ev_62a9a8ff…` |
| XTEA delta -1640531527 = 0x9E3779B9 (`xtea::DELTA`) | `jagex/ByteBuf.java` @ 0x215b | `ev_a06217d6…` |
| Sample started part-way past a ping-pong loop's end reflects back: 2·len − 1 − pos (`Player::seek_offset`) | `jagex/Node_Sub15_Sub2.java` @ 0xf96 | `ev_309a4bda…` |
| Height mode 1 = parent height − base height (layout; why the lobby body grows when the bar's 50 px are taken off) | `jagex/Class484.java` @ 0x2fa | `ev_938babf2…` |
| A hidden component isn't drawn or given input (`method11651` → `hidden`; `Ui::hide_lobby_top_bar`) | `jagex/client.java` @ 0x6dac, `jagex/IComponentDefinitions.java` @ 0x94f1 | `ev_f95d77b4…`, `ev_4246d6f3…` |
| Login music request fades the old song out 2 per tick (`Jukebox::start(.., 2)`) | `jagex/Class339.java` @ 0x29d | `ev_37ce0528…` |
| The login music doesn't loop (`aBool1962 = false`) | `jagex/Class339.java` @ 0x279 | `ev_f59c6e58…` |
| Starting from volume 0 cuts at once: step 10000 (`instr6629`) | `jagex/Class109.java` @ 0x1a8 | `ev_fbef5428…` |
| Player volume starts at 256 (`jukebox::FULL`) | `jagex/Node_Sub15_Sub2.java` @ 0x75 | `ev_0072ed2f…` |
| The client frame can be resized (`main.rs` `window_resizable`) | `jagex/Engine.java` @ 0x2eff | `ev_8aff51c7…` |
| Max screen size 1 caps the game area at 1024 × 768 (`resizable_area`) | `jagex/Class46.java` @ 0x5ab | `ev_dbeee6e1…` |
| A new window size is applied 500 ms after it changed (`Window::follow_frame`) | `jagex/client.java` @ 0x1164a | `ev_490266c7…` |
| A resize layout queues `anObjectArray1271` when a component's size changed (`layout_after_resize`) | `jagex/Class484.java` @ 0x75b | `ev_bcd5be11…` |
| PLAY only acts in game state 0 with no login running (`instr6641`, `world_login`) | `jagex/Class466.java` @ 0x273 | `ev_2f17411a…` |
| World login replies go to `anInt106` while `lobbyStage` is 273 (`Ui::world_login_reply`) | `jagex/FontRenderer_Sub1.java` @ 0x7e8 | `ev_374398a3…` |
| `anInt106` starts at -2 | `jagex/Class9.java` @ 0x53 | `ev_93cee363…` |
| A jingle plays at volume × music volume >> 8 (`Jukebox::jingle`) | `jagex/Huffman.java` @ 0xa47 | `ev_ce7b7628…` |
| Logout resets the login reply code to -2 (`Ui::log_out`) | `jagex/JS5StandardRequester.java` @ 0x284 | `ev_e90fb3e3…` |
| Logout forgets the song: `anInt3223 = -1` (`jukebox.song`) | `jagex/CursorIndexLoader.java` @ 0x435 | `ev_45d716f9…` |
| A new root window unloads the old one (`Ui::unload_root`) | `jagex/QuickChatMessage.java` @ 0x22d9 | `ev_9b9d9e0a…` |
| State 5 is logged out, same music group as the lobby (no login music restart) | `jagex/GameState.java` @ 0x5f | `ev_af718f22…` |
| Unset plane-0 tile height: −noise(x + 932731, y + 556238) × 8 << 2 (`Terrain::decode`) | `jagex/Class329.java` @ 0x126da | `ev_f41e574e…` |
| Unset higher-plane tile height: the plane below − 960 | `jagex/Class329.java` @ 0x127a8 | `ev_d86878c5…` |
| Noise height = (int)(n × 0.3) + 35 (`noise_height`) | `jagex/CutsceneObject.java` @ 0x26f | `ev_dcd236b3…` |
| Base noise n × (n² × 15731 + 789221) + 1376312589 & 0x7fffffff (`random`) | `jagex/Class260.java` @ 0x24c | `ev_18d7cadc…` |
| Colour table brightness 0.7 + (random × 0.03 − 0.015) (`palette::random_brightness`) | `jagex/CutsceneCameraMovement.java` @ 0x551 | `ev_9aab5717…` |
| Packed HSL = (hue >> 2 << 10) + (lightness >> 1) + (saturation >> 5 << 7) (`floors::rgb_to_hsl`) | `jagex/Class5.java` @ 0x73e | `ev_b6ce6b32…` |
| Overlay colour 0xff00ff means none (-1) | `jagex/IdentikitDefinition.java` @ 0x24f | `ev_7d90fc02…` |
| A tile's overlay byte b names definition b − 1 | `jagex/Class329.java` @ 0x6159 | `ev_f61dfb7f…` |
| Full-screen sizes are at least 800 × 600 (`display::modes`) | `jagex/ClanVarDefinitions.java` @ 0x1a6a | `ev_b90f3a5a…` |
| `setPref` refuses only `checkValid` 3 (`Preferences::set`) | `jagex/Preference.java` @ 0x66e | `ev_f0d3fd22…` |
| Scenery shadows refused (3) while textures are off (`check_valid`) | `jagex/SceneryShadowPreference.java` @ 0x2cc | `ev_34e19f1c…` |
| The login reply reaches the window's scripts as `Class9.anInt72` (`ui.login_reply`) | `jagex/FontRenderer_Sub1.java` @ 0x7ca | `ev_01ab9ede…` |
| `SETUP_MESSAGEBOX` only stores a style (`aBool2135`) for the toolkit box | `jagex/Node_Sub15_Sub3.java` @ 0x44a | `ev_c7b17531…` |
| That box is drawn by `Class446.method7447` while a toolkit is timed/switched | `jagex/Class446.java` @ 0x10e7 | `ev_524764d9…` |
| No lobby: "Log In" goes straight to the world login | `com/Loader.java` @ 0x516 | `ev_b9d7138f…` |
| First game packet `REGION` = opcode 85, u16 size (`world::REGION`) | `jagex/ServerProt.java` @ 0xb51 | `ev_2a693c4f…` |
| It starts with this player's 30-bit position (`world::region_payload`) | `jagex/Class197.java` @ 0x39f | `ev_8c360acc…` |
| Then one 16-byte XTEA key per remaining region (`read_region`) | `jagex/MapRegion.java` @ 0x3dee | `ev_8e139ed4…` |
| Lobby window = entity defaults opcode 6 (`Defaults::lobby_window`) | `jagex/EntityDefaults.java` @ 0x9b3 | `ev_4a2220b1…` |
| Game state 0 makes the lobby window the root (`Window::open_lobby`) | `jagex/QuickChatMessage.java` @ 0x25d5 | `ev_8c279a6d…` |
| Lobby reply reads the membership end as an i64 (`encode_lobby_ok`) | `jagex/Login.java` @ 0x657c | `ev_ee836cba…` |
| A lobby login goes straight to game state 0, no first game packet | `jagex/Login.java` @ 0x7734 | `ev_44542255…` |
| `VARP_SMALL` = packet 115, 3 bytes (`server_prot::VARP_SMALL`) | `jagex/ServerProt.java` @ 0xf8d | `ev_ca08565a…` |
| A varbit = var >> start & mask (`Ui::varbit`) | `jagex/PlayerVarProvider.java` @ 0xff0 | `ev_0f5e506c…` |
| Each variable change goes into a 32-entry ring (`VarpChanges::record`) | `jagex/PulseEvent.java` @ 0x417 | `ev_3f05f340…` |
| `FRIEND_COUNT` is −2 while the list isn't loaded (`Social::friend_count`) | `jagex/CS2Interpreter.java` @ 0x34d71 | `ev_b1ad047f…` |
| Membership end in minutes = `MEMBERSHIP_END / 60000` (`USERDETAIL_LOBBY_MEMBERSHIP`) | `jagex/CS2Interpreter.java` @ 0x45bda | `ev_a8f72ecb…` |
| `aLong420` = end − now − 5-byte value (time left counts down from login) | `jagex/Login.java` @ 0x65ca | `ev_f458cc8b…` |
| Varbit opcode 1 = base variable u16, start, end (`cache/varbit.rs`) | `jagex/VarBitDefinitions.java` @ 0x2cf | `ev_e0614b36…` |
| Option n allowed when settings bit n + 1 is set (`menu::enabled`) | `jagex/IFTargetParams.java` @ 0x222 | `ev_3ee13d03…` |
| `IF_OP1` = client packet 96, 8 bytes (`client_prot::IF_OPS`) | `jagex/ClientProt.java` @ 0xc1c | `ev_09bafb3a…` |
| `IF_OP` payload starts with the component id, little-endian (`if_op_payload`) | `jagex/Class350_Sub3.java` @ 0x2df | `ev_8c3f1c42…` |
| Options 5..1 go on the menu last, so option 1 ends on top (`menu::top_option`) | `jagex/Class455.java` @ 0x81f | `ev_b806b3d7…` |
| note volume = velocity² × note volume × patch volume + 1024 >> 11 | `jagex/Node_Sub15_Sub2.java` @ 0x1b70 | `ev_b335199c…` |
| Player table has 2048 slots (`players::MAX_INDEX` = 2047) | `jagex/Class197.java` @ 0x125 | `ev_770f470e…` |
| Player indices run 1..2047; 0 is unused (`World::new` hands out 1..=capacity) | `jagex/Class197.java` @ 0x7cc | `ev_64cc8199…` |
| Reply 7 (world full) has no special case: generic refusal, connection closed | `jagex/Login.java` @ 0x4861 | `ev_44fbb8a0…` |
| `WORLD_LANGUAGE` pushes `CURRENT_LANGUAGE.getValue()` (`ops.rs`: 0) | `jagex/CS2Interpreter.java` @ 0x497f5 | `ev_b11d9911…` |
| The Loader sets language param 31 to 0 | `Loader.java` @ 0xb7c | `ev_3abe306d…` |
| Language 0 is English, so the lobby top bar (906:1, script 6384) uses the English button pictures 10159-10168 | `jagex/Language.java` @ 0x61 | `ev_3b87aa0d…` |
| The world list is asked for again only when the last one is ≥ 1000 ms old (`Worlds::request`) | `jagex/CS2Interpreter.java` @ 0x4475b | `ev_3edf260a…` |
| World number = sent offset + first world number (`world_list::decode`) | `jagex/CutsceneActionType.java` @ 0xe03 | `ev_95cf99ad…` |
| Player count 65535 means offline (-1) | `jagex/Class359.java` @ 0x18e | `ev_7b88d6d5…` |
| Sorting by players ascending puts offline worlds last (-1 counts as 2001) | `jagex/Preference_Sub1.java` @ 0x2dc | `ev_514c10d7…` |
| World sort is the client's quicksort: middle pivot moved to the end, entries comparing ≤ 0 go left (`worlds::quicksort`) | `jagex/Class52.java` @ 0x3a2 | `ev_5120b747…` |

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
279, 283), 144 program notes, so only their samples need loading. The MIDI player
(`client/music/mod.rs`, `Node_Sub15_Sub2`) runs the song's events at their times (division ×
1,000,000 / 22050 per sample), plays notes from the patches with their volume, pan, pitch bend,
vibrato, portamento and envelopes, updates voices every 10 ms, and mixes them
(`client/music/voice.rs`, `Node_Sub15_Sub5`: 8.8 position, linear interpolation, ramps, loops and
ping-pong) into 24-bit then 16-bit at 22,050 Hz. `--render-music 0` saves 60 s of the login
track as WAV. Not ported yet: controller 81 retrigger and the voice budget. Still to do: starting it
at the login screen (`Class339.method6047`, `ProcessorSpecs.method7730` /
`MaterialProp33.method15420`) and playing it through the sound card. Also not yet: the 3D scene behind the window (content type 1337, camera instructions are popped and
ignored), models, rotated sprites, the message box style (`SETUP_MESSAGEBOX` is checked and dropped;
login messages don't need it), key bindings (`aBool1424`), the sprite click mask, and hooks waiting on var/inventory/stat changes (none exist before login).

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
| Accounts | first login creates the account, later logins need the same password (3), already online (5) (server choice) | — |
| World full | no free player index (1..2047, `world_capacity`, 2000 by default) → reply 7 (server choice; the client treats it as any refusal) | `Class197`, `Login.java:316-320` |

**RSA key:** the reference encrypts the password with the public modulus in `Loader.java:25`. The
server needs the matching private exponent to read it, and that isn't in the client. Either put that
private key in `server.cfg` (`login_rsa_exponent` / `login_rsa_modulus`), or run `--gen-rsa` and put
the new modulus into `Loader.java`.

**Not ported yet:** the login packet's later fields (preferences, machine info, cache CRCs, world id),
which the Rust client doesn't send yet, and everything the server must send after login (`Login.java:487+`).

### Dragging and the mouse wheel (login options' volume sliders)
A click on a draggable component starts a drag (`Node_Sub14.method12221`): draggable means
settings depth (`>> 18 & 7`) or whole-screen bit (`>> 23`) or a drag parent set by script
(`instr6214`/`instr6676`, `CS2Interpreter.method3738`). The drag stays inside that parent
(`TexturesPreference.method12880`). Each tick after the hook queues (`EntitySpotAnim.method2827`):
the component goes to mouse − grab point, clamped left/top first to the parent's box; after
`anInt1381` ticks a move past `anInt1380` pixels counts; while held the drag hook
(`anObjectArray1451`) runs, on release the drop hook (`anObjectArray1394`) runs at once with the
position inside the parent (+ its scroll). While dragging, no press/hold hook fires. Drawing
(`IComponentDefinitions.render`): a moved drag is drawn at that position; modes 0 and 2
(`anInt1382`) draw it last, over everything. The login sliders (interface 882, knobs 51/58/66/89:
dead time 2, mode 0) set music 0–255 and sounds 0–127 through scripts 1215/2005. The wheel adds
notches (`anInt7191`, positive towards the user) and queues `anObjectArray1412` with mouse y =
notches on components under the cursor (`client.java:1183`). Not ported: drop targets (server)
and the menu click of a drag that never moved.

### Login retries (`Login.method5018`)
A login tries again up to 3 times (`Class9.anInt104`): when the connection fails, when the server
replies 23, and when a try takes longer than 250 client ticks (5 s) the first time or 2000 ticks
(40 s) after. After the third retry a failure gives -4 and a slow try gives -5 (`client/login.rs`
`attempt`). The reference also switches to its other port on each retry; there is one address here.

### Saved preferences (`preferences.dat`)
`Class190.savePreferences` writes `GamePreferences.encode`: version 27, then 35 bytes in this
order: anti-aliasing (saved), bloom, brightness, build area, sub16, flickering, fog, ground
blending, ground decor, idle animations, light detail, scenery shadows, toolkit default,
particles, remove roofs, max screen size, skyboxes, character shadows, textures, toolkit, 0,
water, screen size, custom cursors, graphics, CPU, sub11, safe mode, sub3, sound, area sound,
voice, music, login music volumes, mono/stereo (`client/preferences.rs` `encode`). The settings
instructions save right after they change something (`SAVES_PREFERENCES` in `cs2/ops.rs`, plus
Auto Setup `instr6852`). `FixedTileStrategy.method12791` reads it back (`decode`, versions 23–27)
and `method13502` puts each value outside its range back to its default (`check`); water, CPU
and the volumes use `< 0 && > n`, never true, so they are never reset. **Reference bug kept:**
the custom cursors byte is written (`GamePreferences.java` @ 0x4735) but not read (@ 0x16b7), so
from graphics on each preference gets the byte of the one before it — after a restart the music
volume is the old voice-over volume and mono/stereo falls back to stereo. Safe mode: on at start
(toolkit back to 0 if it was already on, `IndexLoaders`; `instr6784` reports it) and off again,
saved, once the login window is up (`client.java:2165`). The tools and the self-test don't touch
the file.

### Auto Setup with the graphics card (`ApplyHitmarkCutsceneAction.method14641`)
Auto Setup times each hardware toolkit that may start (`Class383.method6513`): it draws the
entity defaults' profiling model (opcode 2, a mesh from index 7, `cache/mesh.rs` = `RSMesh`
new and old formats) in frames of 136, rows of 1 to 16 at x = 512·(col − row/2),
z = 512·(row + 1), for 1000 ms; the score is meshes per second (no profiling model → 1).
Toolkit 2 (the reference's native software renderer) and 3 (DirectX) can't start here (-1);
toolkit 1 (OpenGL) is timed with macroquad's OpenGL (`client/gpu.rs`). Its score × 1.1 picks the
level (`FontRenderer_Sub1.method13782`: > 20077 → HIGH, > 10021 → MID, > 5002 → LOW, else MIN)
and toolkit 1 stays running; the saved toolkit only changes when the player answers "Yes" to
"Your display has been changed" (`instr6922`). "No" marks the toolkit as failed (`instr6858` →
`Class188.aBool2377`) and Auto Setup runs again without it; with no hardware toolkit left the
CPU picks the level (`ServerEnvironment.method8313`), and toolkit 0 is saved and started.
Ported for that dialog: `DETAIL_ANTIALIASING_DEFAULT`, `instr6770`, `instr6674`, `instr6745`
(start a toolkit: only 0 and 1 start here), `DETAIL_BLOOM` (`Class115.setBloom`), `MES` (chat;
none on the login screen). Tools can give a score with the `opengl n` input step.

### Options panel: CUSTOM settings and full-screen sizes
Every "Advanced" setting the panel shows is ported (`cs2/ops.rs`): getters, the
`DETAILCANMOD_*` / `DETAILCANSET_*` checks (`Preferences::can_modify`, `check_valid`: 1 fine,
2 allowed but turns something else off, 3 refused — e.g. scenery shadows need textures, fog
and water need ground blending, anti-aliasing, bloom and skyboxes need a toolkit with
effects) and the setters (`method5766` … `method452`), each through `setValue` (`setPref`
refuses 3, then `method13502` re-checks all) and saved. `instr5975` is the particle level
(`Class235.anInt2906`, kept equal to the particles preference).
Full-screen sizes (`ClanVarDefinitions.method6825`, `client/display.rs`): the reference lists
the monitor's display modes; here the usual PC sizes up to the main monitor's size (read with
`display-info`) plus that size stand in for them. Kept from the reference: at least 800 × 600,
"max screen size" 2 caps at 800 × 600 and 1 at 1024 × 768, one per size, smallest area first
(`instr6704` count, `instr6404` size n, `instr6643` the one in use: none). Before login the
cache's script answers a full-screen click with "only available to members"
(`PLAYERMEMBER` is 0 until the login reply says otherwise — server side).

### Accounts (server side, this server's own rules)
There is no reference server, so `src/login/accounts.rs` decides: the first login with a name
creates the account; later logins need the same password, else reply 3 (the window shows
"Invalid login or password."); a name already in the game gets reply 5 ("Your account has not
logged out from its last session."). Names match without case or surrounding spaces. Saved in
`data/accounts.txt` (server.cfg `accounts`) as `name salt Whirlpool(salt‖password)`, never the
password; the self-test keeps its account in memory.

### Message box style (`SETUP_MESSAGEBOX`, not drawn)
It styles the "Profiling" / "Loading - please wait" box the reference draws while it times or
switches a toolkit (`Class446.method7447`). Those run inside a script instruction, and this
client can only show a new frame once the instruction is over, so the box would appear after
the work is done; the style is checked and kept out.

### After the login reply: straight to the world (no lobby)
This reference has `Loader.LOBBY_ENABLED = false` (`com/Loader.java:28`), so "Log In" doesn't do
a lobby login (type 19, `MaterialProp35.lobbyLogin`) but goes straight to `worldLogin` (type 16,
lobby stage 273). The lobby and its world list exist in the cache but this client never opens
them. After the reply-2 body (`Login.java` 384-403) a world login waits for the first game
packet (stages 167 → 173 → 156): an ISAAC-scrambled smart opcode (`readEncryptedSmart`, the
server's cipher = seeds + 50), a u16 size, then `REGION` (opcode 85): player positions
(`Class197.method3203`: 30 bits for this player, 18 for every other index) and the map scene
(`MapRegion.loadMapSceneNormal`: map size, centre chunk x/y, force byte, then 4 XTEA ints per
region from (chunk − 6) / 8 to (chunk + 6) / 8). The server sends it (`src/login/world.rs`,
start at 3222, 3218 plane 0 — this server's choice) with the keys of `keys.json` by
`mapsquare`; the client reads it (`client/login.rs` `read_region`). Next: load and draw that map.

### Lobby login (plan step 1 of 6, see ROADMAP.md)
With `lobby = true` (server.cfg, default; the reference has `LOBBY_ENABLED = false`) "Log In"
sends a lobby login (type 19, `MaterialProp35.lobbyLogin`: no reconnect byte, no window size;
game and language bytes). The server answers code 2 with the lobby body (`Login.java:409-450`,
`protocol::encode_lobby_ok`; its values — member, verified, no news server — are this server's
own) and nothing more: unlike a world login there is no first game packet (`Login.java:477`:
login stage 2, game state 0). The client reads the body (`decode_lobby_ok`) and, as
`QuickChatMessage.method6161` does for game state 0, makes the entity defaults' lobby window
(opcode 6, `lobbyWindow`) the root and runs its on-load scripts. Not yet: the lobby connection
stays closed after the reply (step 3), and the lobby's scripts stop at unported instructions
(step 2).

### Lobby window scripts and the lobby connection (plan steps 2–3)
The lobby window (interface 906) runs on-load script 3057, which gives component 906:0 the
every-tick hook 6347. That script builds the lobby (script 1299) only once player variable 2567
isn't 0 — a variable only the server sets. So the lobby connection stays open after the reply
(`client/lobby.rs`) and the server sends `VARP_SMALL` 2567 = 1 right after it
(`src/login/lobby.rs`; the value is this server's own: the script only tests ≠ 0).
Packets (`src/login/server_prot.rs`, generated from `ServerProt.java`, 162 ids with sizes): the
id as an ISAAC-scrambled smart, a u8 length for size −1, a u16 for −2, then the payload.
`VARP_SMALL` (115, size 3): value `readByte`, id `readUnsignedShortLE128` (low byte − 128, then
high). `VARP_LARGE` (8, size 6): value `readIntV2` (bytes 2, 1, 4, 3), id LE+128. The client
reads whatever has arrived each tick without blocking, keeps a two-byte id's first half when a
read splits it, and stores variables in `Ui::varp` for `LOAD_VARP`. Ported for the lobby's
scripts: `LOAD_VARP`, `CC_SETCHILD`, `CC_SETVFLIP`, `CC_SETTRANS`, `IF_SETTEXTFONT`, `instr6232`
(release hook), `instr6222` (option cursor: popped, no custom cursors drawn yet).

### Lobby window: what its scripts read (plan step 2)
With variable 2567 set, script 1299 builds the lobby and script 5954 fades it in (transparency
+2 per tick up to 255, `instr6281` reads it back). Ported for that path, each answering as the
reference does before the server sends the data it's about:
- Player variables (`client/interface/mod.rs`): `LOAD_VARP`; `LOAD_VARPBIT` (index 22,
  `VarBitDefinitions` opcode 1: base variable, start and end bit; value = var >> start &
  (2^(end − start + 1) − 1), `cache/varbit.rs`); hooks waiting on variables (`instr6237`,
  `anObjectArray1397` with its `varps` list): a running count of changes and the last 32 ids
  (`anInt7453`, `anIntArray7379`), recorded once per tick before the input pass; a hook runs
  when one of its variables changed since it last looked, or on any change if it lists none or
  missed more than 32 (`client.java:1520-1543`).
- The lobby reply's fields (`USERDETAIL_LOBBY_*`, membership minutes, `instr6797` = days since
  1970 − 11745), dates (`FORMAT_DATETIME_FROM_MINUTES` "dd-Mmm-yyyy HH:MM UTC", `FROMDATE`
  "dd-Mmm-yyyy", `cs2/calendar.rs`; the reference's `FROMDATE` uses the machine's time zone,
  here UTC), this player's name (`instr6574`), mod and staff levels, membership
  (`PLAYERMEMBER` = the reply's member flag: script 6347 builds the lobby straight away for a
  member when varc 1882 is 1, and shows the subscription offer, hook 1999, otherwise).
- Friends and ignore lists (`interface/social.rs`): "not loaded" (`FRIEND_COUNT` −2,
  `IGNORE_COUNT` −1) until the server sends them; no clan channel (`FIND_AFFINED` 0), no
  friends chat (0 / ""), no chat history (−1 / ""), no private filter (−1), no world list
  (`instr6807` answers "unknown world"), `WORLD_ID` = 177 (`JS5_SOURCE_WORLD`) until a world is
  chosen.
- Component setters and event hooks (`CC_SETOP`, `CC_SETONMOUSEOVER`, `CC_SETMAXLINES`,
  `IF_SETTEXTALIGN`, `instr6235/6240/6687/6708/6247/6250/6260/6261`), and the right-click menu's
  instructions popped until the menu is ported (`instr6076`, `instr6569`, `instr6220`,
  `instr6661` = no menu open, `instr6657` its look).

### Lobby panels (plan step 2)
Script 1299 gives the lobby root (906:0) a hook on player variable 2411 (`instr6237`, script
2354). That hook reads varbit 10242 (variable 2411, bits 0..=2): 0 shows nothing, 1 shows the
main panel (906:32: tabs Player Info, World Select, Friends, Friends Chat, Clan Chat, Options,
and Play), 2 shows 906:40 instead. The server sends 2411 = 1 with 2567 = 1 right after the
lobby reply (`src/login/lobby.rs`; both values are this server's own). A hook set after the
variable changed still runs: its "seen" count starts at 0, so the change is still in the last 32.
Membership in the lobby reply: this server gives a year from each login (`MEMBERSHIP_MS`); the
5-byte field is the time left in ms, which the Player Info panel shows as "Expires dd-Mmm-yyyy".

### Component options and the left click (lobby tabs)
A component's options (`optionNames`, from the cache and `CC_SETOP` / `IF_SETOP`) join the
right-click menu while the mouse is over it (`Class455.iComponentOnIComponent`, options 10..6
then 5..1, each on top) when its settings allow that option (`clickOptionEnabled(n)` = settings
bit n + 1) or it has a `params` hook, and the text isn't blank (`ItemIndexLoader.method7171`).
A left click runs the top entry (`CutsceneAction_Sub10.method14603`): the `params` hook at once
with the option number (hook magic `anInt8051`), then `IF_OPn` to the server if the settings
allow it (`client/interface/menu.rs`). Only the top entry is kept; the menu itself isn't ported.
The lobby tabs (906:214..219, settings 0x2 / 0x402, `params` hook 3059 → script 3060) switch
this way; World Select then waits for the world list ("Please wait...").
Client packets (`src/login/client_prot.rs`, generated from `ClientProt.java`, 104 ids): the id
as one byte plus the client's cipher (the login seeds), the length for sizes −1 / −2, the
payload. `IF_OP1..10` = 96, 27, 68, 9, 72, 19, 23, 21, 22, 81, 8 bytes: component id as a
little-endian int, item u16 (−1 none), slot u16 with 128 added to the low byte. The cache's
first trigger list (`varps`, `decodeScript3`) now reaches the component, so cache hooks waiting
on variables work like script ones. The server reads and logs every client packet by its size.

### Lobby top bar: off by choice (`server.cfg` `lobby_top_bar`)
The bar (Home / Support / Forums / Clans) is component 906:1 of this cache's lobby interface; the
reference client shows it (no script or Java code hides it). It is a custom bar, so this port hides
it by default: `Ui::open_lobby` hides 906:1 and moves the top-level components that sat under it
(y = its height, y mode 0: 906:0 and 906:335) up by its height, and those sized "parent − n"
(height mode 1) grow by it. `lobby_top_bar = true` shows it as the reference does. Tools: input
step `lobby-bar name` opens the lobby with it.

### Login music playing (`Class148`, `MaterialProp33.method15420`, `ProcessorSpecs.method7730`)
Entering the login screens (`GameState.setGameState`: login, lobby) sets the song to the login
track (`Class260.anInt3223 = anInt3228`) and, when the login music volume isn't 0, asks for it
(`Class339.method6047`: state 1, volume target, fade step 2, no loop); else `method6240` (fade
out, nothing next). Each 20 ms tick (`pulseSounds`): state 1 takes `step` off the player volume
while a song plays; once silent (or nothing plays) the song stops and the state becomes 2 if
another is due, else 0 (a song half loaded is dropped). State 2 loads the song and its patches;
when ready the volume becomes the target and it plays once (the login music doesn't loop: only
in-game music is restarted, `Interface.method1623`). Leaving for the world calls `method6240`.
`instr6629` (login music volume): from 0 starts the song at once (`Class109.method1859`, step
10000), to 0 cuts it (`VarBitDefinitions.method3805`), otherwise sets its volume
(`method1491`). Here the song is rendered whole in a background thread at its start volume (the
volume is applied before the mix is clamped, so it decides what clips) and played with macroquad
audio; later volume changes scale the sound by new / rendered volume (`setVolume` is linear,
`v × volume >> 8`). The login track (0) is 290 s long.

### Window size and screen size modes (`UID192.method7373` → `Class507.method8725`)
The client's frame is resizable (`Engine.method4661`: `engineFrame.setResizable(true)`). The
game area (`BASE_WINDOW_WIDTH` × `BASE_WINDOW_HEIGHT`) depends on the screen size mode: 1 is the
fixed `GAME_WIDTH` × `GAME_HEIGHT` (765 × 503); 2 is the window's inside size, at most
800 × 600 when "max screen size" is 2 and 1024 × 768 when it is 1 (`Class46.method935`); either way
`GAME_CANVAS_X` = (window width − area width) / 2, at the top. The client starts in the saved
mode (`IndexLoaders`, `screenSize`, default 2). When the window's size changes, the new size is
applied 500 ms later in the current mode (`client.method11620`, `aLong7307`), and the root
interface is laid out again with resize hooks (`Class516.method8867(true)` → `initSizes`: a
component whose size changed and has `anObjectArray1271` queues it on `PENDING_HOOK_REQUESTS`).
`instr6644` reports the mode (2 when `resizeableScreen`), `instr6645` (`method12933`) switches to 1
or 2 at once. Mouse positions are taken inside the game area (minus `GAME_CANVAS_X`). Not ported:
full screen (mode 3), which falls back to the saved mode as the reference does without it. In
`client/window.rs` (`set_mode`, `follow_frame`) and `interface/layout.rs` (`layout_after_resize`).

### Play: from the lobby to the world (`Class466.method7777` → `Class78.method1384`)
The lobby's PLAY runs script 2727 (`instr6589`: the flags of the game connection's world in the
world list, 0 when unlisted) and 3062 (`instr6641`): in game state 0 with no login running, game
state 7, which starts a world login with the name and password of the last login
(`Class9.aString99` / `aString102`, `ParticleArchive1Def.worldLogin`: `lobbyStage` 273, the
game connection). Its reply codes go to `Class9.anInt106` (`method13784` with `lobbyStage` 273),
which the lobby reads with `instr6305` (-2 none, -3 connecting); the login window's code is a
separate one (`anInt72`, `lobbyStage` 140). `instr6806` picks the world (`Class62.setGameHost`).
Details some replies carry stay at their start values (`instr6774` -1, `instr6720` -1,
`instr6700` 0, `instr6101` 0). The play jingle (`SOUND_JINGLE` 349 → `Huffman.playSoundJingle`)
plays from index 11 at 255 × music volume >> 8, cutting the login music. Server: someone in the
lobby may also log in to the world (lobby and world are separate places for reply 5).

### Logging out from the lobby (`instr6623` → `CursorIndexLoader.logout(false)`)
The lobby's close button runs `instr6623`: every connection is flushed and closed, both login
reply codes go back to -2 (`JS5StandardRequester.method5558`), the music fades out and the song
is forgotten (`Class358.method6240`, `anInt3223 = -1`), the saved name and password are cleared
(`BloomPreference.method12708`), and game state 5 opens the login window again
(`Class348.method6175`). Lobby (0) and login (5) are in the same group of states, so the login
music doesn't start again. Opening a new root window (the lobby, `QuickChatMessage.method6161`,
or the login window) unloads the old root and every interface open inside it
(`Class337.unloadInterface`, `Class351.closeChildren(.., true, ..)`), so the login window comes
back fresh, not with the old login's spinner (`Ui::unload_root`).

### Map terrain (`m{x}_{y}` in index 5; `Class329.decodeTileMasks`)
Map archives are found by name (`Index.getArchiveId`: lower-case name, hash h × 31 + byte):
`m{x}_{y}` is a region's terrain, `l{x}_{y}` its locations (XTEA, the keys `REGION` sends). The
terrain is 4 planes × 64 × 64 tiles, plane first, then x, then y; each tile is a run of opcodes:
0 ends it (height from noise on plane 0, else the plane below − 960), 1 ends it with a height
byte h (1 counts as 0; plane 0: −h × 8 << 2, else the plane below − (h × 8 << 2)), 2..=49 an
overlay byte with shape (op − 2) / 4 and rotation (op − 2) & 3, 50..=81 the settings op − 49,
82 and up the underlay op − 81. The noise height (`CutsceneObject.calculateTileHeights`) of
absolute tile (x + 932731, y + 556238) is three octaves of smoothed value noise
(`Class159.method2735` with cosine interpolation `Class430.method7220` over `Trig.COSINE`,
`Class241.method4151`, `Class260.method4632`), × 0.3 + 35, clamped to 10..=60. Region 50,50 (the
arrival): plane 0 heights −1472..0, 1874 overlay tiles, 550 blocked.

### Floor definitions and the colour table
Underlays are index 2 archive 1 and overlays archive 4, file n = definition n; a tile's underlay
or overlay byte b names definition b − 1 (`Class329`: `getUnderlayDef(id − 1)`). Underlay
(`UnderlayDef`): 1 rgb (u24), 2 texture (u16, 65535 none), 3 scale, 4 and 5 flags. Overlay
(`OverlayDef`): 1 colour (u24 RGB → packed HSL by `Class5.method294`, 0xff00ff = none), 2/3
texture, 7 secondary colour, 9/20/22 u16, 11/14/16/21 u8, 13 u24 water colour, 5/8/10/12 flags
(`cache/floors.rs`). Packed HSL is 6 bits hue, 3 saturation, 7 lightness; `method294` lowers the
saturation of light colours (lightness > 179, 192, 217, 243: >> 1, 2, 3, 4). The colour table
(`CutsceneCameraMovement.method1365`, `client/palette.rs`) turns packed HSL into RGB: hue + 1/128,
saturation + 1/16, standard HSL, each channel to the power of a brightness picked once at random,
0.7 ± 0.015. `--map-terrain` now colours its picture with these.
