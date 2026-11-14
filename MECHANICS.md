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
| A hidden component isn't drawn or given input (`method11651` → `hidden`; `Ui::hide_top_bar`) | `jagex/client.java` @ 0x6dac, `jagex/IComponentDefinitions.java` @ 0x94f1 | `ev_f95d77b4…`, `ev_4246d6f3…` |
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
| Underlays are config archive 1 | `jagex/SharedConfigsType.java` @ 0x38 | `ev_c1a20725…` |
| Overlays are config archive 4 | `jagex/SharedConfigsType.java` @ 0x6a | `ev_7bae4b23…` |
| A region's terrain archive is named `m{x}_{y}` (`map_terrain::archive_name`) | `jagex/MapRegion.java` @ 0x4504 | `ev_c6b2609e…` |
| Archive names hash as (h << 5) − h + byte (`map_terrain::name_hash`) | `jagex/AbstractRenderer.java` @ 0xaa7 | `ev_2cd96b68…` |
| Blended underlay colour = method6348(hue × 256 / chroma, sat / count, light / count) (`ground::blend`) | `jagex/Class329.java` @ 0x34e1 | `ev_05f8546a…` |
| Underlay chroma for lightness > 0.5: sat × (1 − light) × 512 (`Underlay::split_colour`) | `jagex/UnderlayDef.java` @ 0x6a1 | `ev_5a20f14d…` |
| Scene regions run (chunk ∓ sizeX >> 4) / 8 (`ground::regions`) | `jagex/MapRegion.java` @ 0x411e | `ev_c3f39f92…` |
| Camera pitch starts at `camAngleX` 1052 (stand-in view) | `jagex/client.java` @ 0x2d49 | `ev_48f7d72f…` |
| Shape 0 with no overlay becomes 12; overlay faces per shape `OVERLAY_FACE_COUNT` (`tile_faces`) | `jagex/Class329.java` @ 0x6232, 0x45 | `ev_1a64fd12…`, `ev_da639a47…` |
| Ground point lightness × 74 / 128 before the colour table (`lit`) | `jagex/OpenGLGround.java` @ 0x39e5 | `ev_c6424a2e…` |
| Corner normal (dx, −2 × 512, dy) normalised (`Ground::normals`) | `jagex/OpenGLGround.java` @ 0x9a6 | `ev_6846ed28…` |
| Ambient (0.7 + brightness × 0.1 + region random) × 1.1523438 (`Light::scene`) | `jagex/Class239.java` @ 0x1007 | `ev_b32b2b91…` |
| Rotation 1 turns a point to (y, 512 − x) (`tile_faces`) | `jagex/Class329.java` @ 0x72b6 | `ev_bcca2cd0…` |
| An overlay with neither colour is left out (`Ground::load`) | `jagex/Class329.java` @ 0x62d5 | `ev_fd2adf46…` |
| Sun strength 0.69921875 lit (`Light::strength`) | `jagex/Atmosphere.java` @ 0x2b3 | `ev_9c7a50e6…` |
| Point strength ambient + d × (lit if d > 0 else shaded) (`Light::strength`) | `jagex/OpenGLGround.java` @ 0x4626 | `ev_b148a63a…` |
| Ground blending preference picks `method5846` (`Ground::load`) | `jagex/MapRegion.java` @ 0x8e7c | `ev_3e78593f…` |
| Underlay diagonal: rotation 1 when abs(sw − ne) < abs(se − nw) (`Floors::diagonal`) | `jagex/Class329.java` @ 0x1232c | `ev_51fad3d8…` |
| Underlay point colour mixed from the corners (`mix`) | `jagex/Class329.java` @ 0x10a47 | `ev_1a840fed…` |
| Overlay priority (opcode 11) << 8 \| id (`Overlay::decode`) | `jagex/OverlayDef.java` @ 0x24f | `ev_d9cf55cc…` |
| A side neighbour takes a point when the priority there ≤ its own (`Around::take`) | `jagex/Class329.java` @ 0xabaa | `ev_505c12c9…` |
| Shape 0 / 12 with two edges at a corner becomes 13 / 14 (`Floors::blended`) | `jagex/Class329.java` @ 0x1187e | `ev_ef754122…` |
| A side gets an edge when its corner points share no overlay (`Floors::blended`) | `jagex/Class329.java` @ 0x1158b | `ev_7a59347f…` |
| Underlay edge point takes a claiming overlay's colour (`Floors::blended`) | `jagex/Class329.java` @ 0xffb3 | `ev_4a355819…` |
| HSL mix by t / 128, parts apart (`mix`) | `jagex/Class371.java` @ 0x12c | `ev_654519a0…` |
| Overlay opcode 12 blends (`Overlay::decode`) | `jagex/OverlayDef.java` @ 0x747 | `ev_fa069941…` |
| Locations kept only strictly inside the scene (`locations::load`) | `jagex/Class329_Sub1.java` @ 0x4869 | `ev_980f9b35…` |
| Location plane = position >> 12 (`map_locations::decode`) | `jagex/Class329_Sub1.java` @ 0x4769 | `ev_e4e5d250…` |
| Object definitions: archive id >>> 8, file id & 0xff (`Assets::object`) | `jagex/SharedConfigsType.java` @ 0x8f | `ev_712f1e30…` |
| Locations file decrypted with the region's key (`Assets::locations`) | `jagex/MapRegion.java` @ 0x72a3 | `ev_01378820…` |
| HSL hue with red highest: cb + 20480 or 4096 − cg (`to_hsl`) | `jagex/MaterialProp17.java` @ 0x990 | `ev_dc1ced40…` |
| Warp angle v >> 4 & 0xff for one channel (`Render::mono`) | `jagex/MaterialProp19.java` @ 0x217 | `ev_f692e4c5…` |
| Invert 4096 − v (`Render::mono`) | `jagex/MaterialProp22.java` @ 0x45a | `ev_c6863b72…` |
| Emboss scale × width × 32 >> 12 (`Render::mono`) | `jagex/MaterialProp32.java` @ 0x303, `Class316.java` @ 0x8d6 | `ev_78c96345…`, `ev_bd18a825…` |
| Range span high − low (`Settings::finish`) | `jagex/MaterialProp30.java` @ 0x1432 | `ev_5d807de3…` |
| Mix (4096 − t)·b + t·a >> 12 (`mix`) | `jagex/MaterialProp21.java` @ 0x3c8 | `ev_f3562413…` |
| Range: span × v >> 12 + low (`Render::mono`) | `jagex/MaterialProp30.java` @ 0x1704 | `ev_f9a78cd5…` |
| Warp angle v × 255 >> 12 & 0xff for three channels (`Render::rgb`) | `jagex/MaterialProp19.java` @ 0x5cf | `ev_eb8c9b50…` |
| Emboss table 255 / √(... / 65535.0f) (`emboss_table`) | `jagex/MaterialProp32.java` @ 0x12d | `ev_e3abe334…` |
| HSL blue l × −680551435 leftover (`from_hsl`) | `jagex/MaterialProp17.java` @ 0x10d8 | `ev_db50b9aa…` |
| Mirror reads row height − 1 − y (`Render::mono`) | `jagex/MaterialProp9.java` @ 0x15f | `ev_c3016306…` |
| Operation 0's value (byte << 12) / 255 (`Settings::read`) | `jagex/MaterialProp0.java` @ 0x218 | `ev_1ed5045b…` |
| Operation 1's red (rgb & 0xff0000) >> 12 (`Settings::finish`) | `jagex/MaterialProp1.java` @ 0x14b | `ev_eb100ed1…` |
| Blur factor 65536 / (2r + 1) (`Render::mono`) | `jagex/MaterialProp5.java` @ 0x182 | `ev_c7f8db32…` |
| Black pixels get alpha 0 (`texture::render`) | `jagex/MaterialDefinitions.java` @ 0x2058 | `ev_600f4981…` |
| Default curve (0, 0)–(4096, 4096) (`Settings::finish`) | `jagex/MaterialProp8.java` @ 0x683 | `ev_8de2ebdb…` |
| Texture rows written right to left (`texture::render`) | `jagex/MaterialDefinitions.java` @ 0x1cf8 | `ev_1ff6f240…` |
| Brightness table pow(i / 255, power) × 255 (`brightness_table`) | `jagex/LRUCache.java` @ 0x2dd | `ev_5b6fb1a9…` |
| x as (i << 12) / size (`texture::render`) | `jagex/Class316.java` @ 0x855 | `ev_2fff77a3…` |
| Blend mode 6 overlay (`blend`) | `jagex/MaterialProp7.java` @ 0x952 | `ev_86a8fac8…` |
| Curve mode 1 cosine (`curve`) | `jagex/MaterialProp8.java` @ 0xa79 | `ev_e73fe900…` |
| Ramp channels: red (c & 0xff0000) >> 12 (`Render::rgb`) | `jagex/MaterialProp10.java` @ 0x352 | `ev_405ecff0…` |
| `getFile(id)` reads archive 0's file id with one archive (`texture_stats`) | `jagex/Index.java` @ 0x1c73 | `ev_44886650…` |
| A material ends with its three outputs (`Material::decode`) | `jagex/MaterialDefinitions.java` @ 0x925 | `ev_aefa01ee…` |
| Operation 29's shape type 1 is 20 bytes (`skip_setting`) | `jagex/Class514.java` @ 0x2f0 | `ev_7b5e39e6…` |
| Operation 10's colour points are 5 bytes each (`skip_setting`) | `jagex/MaterialProp10.java` @ 0x3628 | `ev_5a03f015…` |
| Texture table is index 26 (`texture_table`) | `jagex/IndexLoaders.java` @ 0x2627 | `ev_ee11f3e1…` |
| Ground mesh when its byte is 0 (`texture_table`) | `jagex/ImageIndexLoader.java` @ 0x37b | `ev_722fab26…` |
| Each operation: a byte, then its kind (`Material::decode`) | `jagex/MaterialProperty.java` @ 0x6f4 | `ev_16b99460…` |
| Noise table: the swapped byte copied to 511 − i (`permutation`) | `jagex/Class316.java` @ 0x687 | `ev_14d2b000…` |
| Bounded draw rejects at MIN_VALUE − (2³² mod n) (`JavaRandom::below`) | `jagex/Class476.java` @ 0x2be | `ev_fbb1e710…` |
| Fade curve a = 6t − 61440 (`fade`) | `jagex/Class370.java` @ 0x41e | `ev_c2f176ef…` |
| Octave strength (short)(int)(pow(p / 4096f, i) × 4096) (`noise`) | `jagex/MaterialProp34.java` @ 0x1ee0 | `ev_0831861c…` |
| Last octave centred (v >> 1) + 2048 (`Render::noise`) | `jagex/MaterialProp34.java` @ 0x10b7 | `ev_25d8eb48…` |
| Cell distance mode 1: dy² + dx² >> 12 (`cell_distance`) | `jagex/MaterialProp15.java` @ 0x729 | `ev_c7e3933d…` |
| Cell offsets method7931(random, jitter) as shorts (`Settings::finish`) | `jagex/MaterialProp15.java` @ 0x147d | `ev_29bad008…` |
| Operation 36 renders at 64 when half size, else 128 (`pictures`) | `jagex/MaterialPropTexture.java` @ 0x843 | `ev_a7fab7a1…` |
| Operation 36's texture at brightness 1.0 (`pixels`) | `jagex/MaterialPropTexture.java` @ 0x88a | `ev_92cf0ee8…` |
| Inner texture: alpha 255 where the colour isn't 0 (`pixels`) | `jagex/MaterialDefinitions.java` @ 0x163a | `ev_0b10eb56…` |
| Sprite row h·y / height when the heights differ (`Render::rgb`) | `jagex/MaterialPropSprite.java` @ 0x59b | `ev_f230b105…` |
| Tiled sprite row starts at (y % h) × h (`Render::rgb`) | `jagex/MaterialPropSpriteSub18.java` @ 0x176 | `ev_1295f0c4…` |
| Half size when its byte is 1 (`texture_table`) | `jagex/ImageIndexLoader.java` @ 0x44c | `ev_f370dc67…` |
| Sprite frame padding biggest width − width − minX (`Picture::of_sprite`) | `jagex/SpriteDefinitions.java` @ 0x701 | `ev_ae43b1fd…` |
| Position hash starts at n = 57y + x (`hash`) | `jagex/MaterialProp13.java` @ 0x390 | `ev_bc777074…` |
| Position hash 4096 − ((... + 1376312589) & 0x7fffffff) / 262144 (`hash`) | `jagex/MaterialProp13.java` @ 0x3cb | `ev_c69c350f…` |
| Tile row (y % tileH)·height / tileH (`Render::rgb`) | `jagex/MaterialProp20.java` @ 0x693 | `ev_34d85822…` |
| Tile column (x % tileW)·width / tileW (`Render::rgb`) | `jagex/MaterialProp20.java` @ 0x8c1 | `ev_3ef26113…` |
| Tiles across default 4 (`Settings::new`) | `jagex/MaterialProp20.java` @ 0x50 | `ev_b2d8d932…` |
| Shapes clipped to width − 1, height − 1 (`shapes::draw`) | `jagex/MaterialProp29.java` @ 0x648 | `ev_9f27a7bf…` |
| Row cache keeps every row when anInt7668 is 255 (`Render::canvas`) | `jagex/MaterialProperty.java` @ 0x1bb | `ev_a00e7cfb…` |
| Rectangle fill from x1 + width to x2 − width (`Canvas::rect`) | `jagex/HitbarDefinitions.java` @ 0x470 | `ev_da4ce2e1…` |
| Clipped line slope (dy << 12) / dx (`Canvas::line`) | `jagex/QuickChatMessage.java` @ 0x1906 | `ev_46e5383f…` |
| Curve t every 128 of 4096 (`Canvas::curve`) | `jagex/Engine.java` @ 0x976 | `ev_bbe673f0…` |
| Equal radii draw a circle (`Canvas::ellipse`) | `jagex/Class299.java` @ 0x1d5f | `ev_bc7864f1…` |
| Clipped circle notes the inner x after its step (`Canvas::circle_clipped`) | `jagex/Class60.java` @ 0xd9a | `ev_17d52fb8…` |
| Threshold: 4096 within low..=high, else 0 (`Render::mono`) | `jagex/MaterialProp26.java` @ 0x2a2 | `ev_26b7e6ef…` |
| Slope normal length (int)(√(... + 4096) / 4096f) × 4096) (`Render::mono`) | `jagex/MaterialProp35.java` @ 0x535 | `ev_f4d3accd…` |
| Slopes: 4096 − 16777216 / length (`Render::mono`) | `jagex/MaterialProp35.java` @ 0x58d | `ev_5df5aa85…` |
| Scratch count default 2000 (`Settings::new`) | `jagex/MaterialProp38.java` @ 0x7e | `ev_53cf6866…` |
| Scratch start 1024 − (method7931(4096) >> 2) (`Render::scratches`) | `jagex/MaterialProp38.java` @ 0xc85 | `ev_9db02d12…` |
| Scratch value start + 2048 / run per pixel + 1024 (`Render::scratches`) | `jagex/MaterialProp38.java` @ 0xdb1 | `ev_48a153e8…` |
| Shallow scratch written at [major][minor] (`Render::scratches`) | `jagex/MaterialProp38.java` @ 0xef6 | `ev_baf1cbeb…` |
| Stripes default 10 (`Settings::new`) | `jagex/MaterialProp27.java` @ 0x50 | `ev_9396771c…` |
| Stripe on part step·width >> 12 (`stripes`) | `jagex/MaterialProp27.java` @ 0x903 | `ev_68080c20…` |
| Extra stripe end = first end + 4096 (`stripes`) | `jagex/MaterialProp27.java` @ 0xa02 | `ev_360b9292…` |
| Diagonal stripes ((x − (4096 − y)) >> 1) + 2048 (`Render::mono`) | `jagex/MaterialProp27.java` @ 0x38c | `ev_4bf7cacd…` |
| Bricks seeded with the row count (`bricks`) | `jagex/MaterialProp4.java` @ 0xd0f | `ev_be4d5260…` |
| Mortar default 81 (`Settings::new`) | `jagex/MaterialProp4.java` @ 0x110 | `ev_846d6605…` |
| Mortar margin = mortar / 2 (`bricks`) | `jagex/MaterialProp4.java` @ 0xd40 | `ev_00c00525…` |
| Brick shade 4096 − method7931(range) (`bricks`) | `jagex/MaterialProp4.java` @ 0x11d4 | `ev_a1bde15d…` |
| Even rows shift +offset, odd −offset (`Render::mono`) | `jagex/MaterialProp4.java` @ 0x53a | `ev_e4a01044…` |
| Tint keeps the level only for a grey (`Render::rgb`) | `jagex/MaterialProp11.java` @ 0x350 | `ev_3e7d0104…` |
| Round wave × frequency × π as a double (`Render::mono`) | `jagex/MaterialProp12.java` @ 0x377 | `ev_7cada391…` |
| Wave position v −= v & −4096 (`Render::mono`) | `jagex/MaterialProp12.java` @ 0x3d0 | `ev_bf7bd3d3…` |
| Triangle wave (2048 − abs(v − 2048)) << 1 (`Render::mono`) | `jagex/MaterialProp12.java` @ 0x51a | `ev_c7d60522…` |
| Sine wave (sin[v >> 4 & 0xff] + 4096) >> 1 (`Render::mono`) | `jagex/MaterialProp12.java` @ 0x41e | `ev_abf49290…` |
| Tint default 4096 (`Settings::new`) | `jagex/MaterialProp11.java` @ 0x50 | `ev_b709c47e…` |
| Block bevel (width / 8)·bevel >> 12 (`blocks`) | `jagex/MaterialProp28.java` @ 0x1cc5 | `ev_39e64817…` |
| Block shade 4096 − method7931(range) (`block`) | `jagex/MaterialProp28.java` @ 0x2bc | `ev_2c275666…` |
| Block edge bevel − method7931(randomness·bevel >> 12) (`block`) | `jagex/MaterialProp28.java` @ 0x345 | `ev_01b3882a…` |
| Top height ≤ 1 draws nothing (`blocks`) | `jagex/MaterialProp28.java` @ 0x1c6b | `ev_b55a63d5…` |
| Each block row starts at method7931(width) (`blocks`) | `jagex/MaterialProp28.java` @ 0x3242 | `ev_b406e810…` |
| Grey (b + g + r) / 3 (`Render::mono`) | `jagex/MaterialProp24.java` @ 0x20c | `ev_b29e0a19…` |
| Face texture u16 − 1 (`Mesh::new_format`) | `jagex/RSMesh.java` @ 0x5275 | `ev_6a58466f…` |
| Face mapping byte − 1 (`Mesh::new_format`) | `jagex/RSMesh.java` @ 0x532c | `ev_4a99ff00…` |
| Mapping scale 9 bytes from version 15 (`mappings`) | `jagex/RSMesh.java` @ 0x426f | `ev_1cf648da…` |
| Kind 2 mapping colours from the sixth section (`mappings`) | `jagex/RSMesh.java` @ 0x7167 | `ev_5701217…` |
| Old format mapping = render type >> 2 (`Mesh::old_format`) | `jagex/RSMesh.java` @ 0xa081 | `ev_09d129ef…` |
| Upscale leaves kind 1 scale z (`Mesh::upscale`) | `jagex/RSMesh.java` @ 0xb290 | `ev_fb4c5cca…` |
| Planar u over (e2 × normal) · e1 (`planar`) | `jagex/MeshRasterizer_Sub1.java` @ 0x4069 | `ev_5a0afd4a…` |
| Ground texture dropped with textures off (`Textures::used`) | `jagex/MeshRasterizer_Sub1.java` @ 0x33f5, `Class94.java` @ 0x681 | `ev_99bd029f…`, `ev_f1a66421…` |
| Textured face grey = ambient × 131586 (`textured_colour`) | `jagex/MeshRasterizer_Sub1.java` @ 0x15497 | `ev_0fa3c353…` |
| Texture brightness re-packs the channels (`textured_colour`) | `jagex/MeshRasterizer_Sub1.java` @ 0x15818 | `ev_5370b1ff…` |
| Mirroring swaps corners 0 and 2 (`Model::mirror`) | `jagex/MeshRasterizer_Sub1.java` @ 0xe577 | `ev_78127650…` |
| Projection turn cos(rotation × 0.024543693) (`projection_matrix`) | `jagex/MeshRasterizer.java` @ 0x1b2e | `ev_1eeb0e34…` |
| Projection tilt y = dirY / 32767 (`projection_matrix`) | `jagex/MeshRasterizer.java` @ 0x1d23 | `ev_ead51700…` |
| Mapping centre (min + max) / 2 (`projections`) | `jagex/MeshRasterizer.java` @ 0x13b3 | `ev_9c635289…` |
| Cylinder u = atan2(x, z) / 6.2831855 + 0.5 (`cylinder`) | `jagex/MeshRasterizer.java` @ 0x8017 | `ev_f34aebe4…` |
| Sphere v = asin(y / len) / π + 0.5 + shift (`sphere`) | `jagex/MeshRasterizer.java` @ 0x61c6 | `ev_858788c3…` |
| Faces with a skipTriangles texture not drawn (`Model::lit`) | `jagex/MeshRasterizer_Sub1.java` @ 0xde5 | `ev_b5429da9…` |
| Overlay texture scale u16 << 2 (`Overlay::decode`) | `jagex/OverlayDef.java` @ 0x670 | `ev_2c74860a…` |
| Underlay texture scale u16 << 2 (`Underlay::decode`) | `jagex/UnderlayDef.java` @ 0x8e1 | `ev_67c75511…` |
| Ground texture off: no texture at scale 128 (`Ground::light`) | `jagex/OpenGLGround.java` @ 0x8ecc | `ev_1aa23795…` |
| Ground keeps textures when the preference is 1 (`World::build`) | `jagex/MapRegion.java` @ 0x8ee4 | `ev_065982f3…` |
| Ground texture coordinates × 1 / scale (`Ground::light`) | `jagex/Node_Sub3.java` @ 0x19e0 | `ev_a5f7b01d…` |
| Ground shadow factor skipped for effect 4 (`material_colour`) | `jagex/Node_Sub3.java` @ 0x60f | `ev_36387724…` |
| Underlay corner takes its corner tile's texture (`Floors::blended`) | `jagex/Class329.java` @ 0x102e4 | `ev_12443070…` |
| Texture slide ((t % 128000) / 1000) × speed / 64 % 1 (`slide`) | `jagex/OpenGLRenderer.java` @ 0x11538 | `ev_f14bbc42…` |
| Slide is a texture-matrix translation (`REPEAT_FRAGMENT` `Offset`) | `jagex/OpenGLRenderer.java` @ 0x10ea6 | `ev_9e7a7c74…` |
| Clock = (int) Utils.time() each frame (`Scene::draw`) | `jagex/client.java` @ 0x10565 | `ev_c05846a1…` |
| Texture speeds are signed bytes (`texture_table`) | `jagex/ImageIndexLoader.java` @ 0x955 | `ev_c01a6602…` |
| Colour key red kept as (c & 0xff0000) << 4 (`Settings::read`) | `jagex/MaterialProp25.java` @ 0xbda | `ev_7ef5af7b…` |
| Colour key blue (c & 0xff) >> 12, always 0 (`Settings::read`) | `jagex/MaterialProp25.java` @ 0xc4e | `ev_1742ab47…` |
| Frame inner edge default 585 (`Settings::new`) | `jagex/MaterialProp14.java` @ 0x50 | `ev_6ee612fc…` |
| Weave thread position (x % (4096 / across))·across (`Render::mono`) | `jagex/MaterialProp16.java` @ 0x35d | `ev_983a2546…` |
| Lattice half-width from cos[i·255 >> 12 & 0xff] (`Render::mono`) | `jagex/MaterialProp37.java` @ 0x963 | `ev_14194524…` |
| Kaleidoscope angle (float) atan2(x − 2048, y − 2048) (`Render::kaleidoscope`) | `jagex/MaterialProp23.java` @ 0x5965 | `ev_76f45b94…` |
| Kaleidoscope source masked to the picture (`Render::kaleidoscope`) | `jagex/MaterialProp23.java` @ 0x5e74 | `ev_3e200838…` |
| Inputs read right after each operation (`Material::decode`) | `jagex/MaterialDefinitions.java` @ 0x566 | `ev_a817808b…` |
| Operation 34 lists its curve when the short is negative (`skip_setting`) | `jagex/MaterialProp34.java` @ 0x1b1c | `ev_e31a248d…` |
| Flood skips planes with `tileMasks & 8` (`Roofs::flood`) | `jagex/JS5Manager.java` @ 0xd82 | `ev_d56c2570…` |
| Roof edges 19 and 21 face the corner orientations (`edge_orientation`) | `jagex/Class125.java` @ 0x30a | `ev_983e2aef…` |
| Roof mode 2 does nothing on plane 3 (`Roofs::compute`) | `jagex/QuickchatMessageLoader.java` @ 0x4e6 | `ev_1ba1d583…` |
| Roof mode 1 stops after 512 floods (`Roofs::compute`) | `jagex/Class17.java` @ 0x25b | `ev_db346485…` |
| Scenery type 21 compared as 19 (`locations::scenery`) | `jagex/JS5Manager.java` @ 0x18a8 | `ev_8e8e139f…` |
| Roofs: the camera line only with pitch below 2560 (`Roofs::compute`) | `jagex/QuickchatMessageLoader.java` @ 0xc66 | `ev_3fab4df9…` |
| Hidden: plane ≥ player plane + 1 and marked (`Scene::show`) | `jagex/SceneObjectManager.java` @ 0x7e28 | `ev_95a5b633…` |
| Flood west neighbour codes 0x12, 0xd3, 0x13 (`Roofs::flood`) | `jagex/JS5Manager.java` @ 0x250f | `ev_9bc3160c…` |
| Bridge tiles drop their objects a plane (`Ground::slot`) | `jagex/SceneObjectManager.java` @ 0x1601 | `ev_e7473600…` |
| Corner walls face 16, 32, 64, 128 (`wall_orientation`) | `jagex/WallObjectGraphNode.java` @ 0xfd | `ev_d0b9acd1…` |
| Roof mode follows the preference at game start (`World::build`) | `jagex/Class190.java` @ 0x20a1 | `ev_db316f10…` |
| The ground is built for every plane (`Ground::load`) | `jagex/Class329.java` @ 0x2930 | `ev_49a9c8b1…` |
| Fitting 4 / 5 need the ground above and the model inside (`Model::contour`) | `jagex/MeshRasterizer_Sub1.java` @ 0xee3c | `ev_f355310e…` |
| The ground above is the next plane, none on plane 3 (`Model::contour`) | `jagex/GraphNode_Sub1_Sub1_Sub6.java` @ 0xe99 | `ev_f6581ff4…` |
| Model height span max y − min y (`Model::contour`) | `jagex/MeshRasterizer_Sub1.java` @ 0xf59d | `ev_dc6a119b…` |
| One blended colour table for all planes (`Ground::load`) | `jagex/Class329.java` @ 0x27c4 | `ev_1a90d775…` |
| Fitting mode 4: above − model + span (`Model::contour`) | `jagex/MeshRasterizer_Sub1.java` @ 0xf61c | `ev_e829274c…` |
| Fitting mode 5: stretched between the planes (`Model::contour`) | `jagex/MeshRasterizer_Sub1.java` @ 0xf8aa | `ev_a3c75229…` |
| Camera heights on the player's plane (`Camera::process`) | `jagex/Isaac.java` @ 0x4bb | `ev_8e87b73c…` |
| A location sits on its own plane's ground (`locations::scenery`) | `jagex/Class329_Sub1.java` @ 0x4a6a | `ev_05ff3419…` |
| Diagonal decoration offset table (1, −1, −1, 1) (`DIAGONAL_OFF`) | `jagex/Class329_Sub1.java` @ 0x125 | `ev_f9532032…` |
| Diagonal decoration stands off by displacement / 2 + 1 (`pieces`) | `jagex/Class329_Sub1.java` @ 0x9ae0 | `ev_35713701…` |
| The tile's wall is its primary wall slot (`locations::scenery`) | `jagex/SceneObjectManager.java` @ 0x12f9 | `ev_6db66d46…` |
| Inside diagonal decoration at ((rotation + 2) & 3) + 4 (`pieces`) | `jagex/Class329_Sub1.java` @ 0x9f3b | `ev_62082e18…` |
| Animated or transforming objects are dynamic (`ObjectDef::decode`) | `jagex/ObjectDefinition.java` @ 0x3a69 | `ev_14c14441…` |
| Straight decoration offset table (1, 0, −1, 0) (`STRAIGHT_OFF`) | `jagex/Class329_Sub1.java` @ 0xc1 | `ev_5c162a45…` |
| Straight decoration stands off by the wall's displacement + 1 (`pieces`) | `jagex/Class329_Sub1.java` @ 0x95e7 | `ev_8a1251a0…` |
| Whole corner's second piece at (rotation + 1) & 3 (`pieces`) | `jagex/Class329_Sub1.java` @ 0x7479 | `ev_0112ac7d…` |
| Wall decorations 4..=8 use the type-4 models (`pieces`) | `jagex/Class485.java` @ 0xcb | `ev_f8e62bc3…` |
| `decorDisplacement` = opcode 28 << 2 (`ObjectDef::decode`) | `jagex/ObjectDefinition.java` @ 0x16c8 | `ev_f2c51f40…` |
| Decoration offset applied when drawn (`locations::scenery`) | `jagex/GraphNode_Sub1_Sub4_Sub1.java` @ 0x751b | `ev_866c52ec…` |
| Object model ambient 64 + definition's (`Model::build`) | `jagex/ObjectDefinition.java` @ 0x3dc0 | `ev_78b35bb2…` |
| Flat face corner light: normal × 1/384 (`Light::model_strength`) | `jagex/MeshRasterizer_Sub1.java` @ 0x2fc7d | `ev_46319b21…` |
| Ground fitting mode 1: + ground height − model height (`Model::contour`) | `jagex/MeshRasterizer_Sub1.java` @ 0xefb1 | `ev_df355f0f…` |
| Merged meshes share a vertex only at the same x, y, z (`Mesh::merge`) | `jagex/RSMesh.java` @ 0x3224 | `ev_068e2fe2…` |
| Face normal halved until within ±8192 (`face_normal`) | `jagex/MeshRasterizer_Sub1.java` @ 0x2b0f | `ev_c8ba18a8…` |
| Object model contrast 850 + definition's (`Model::build`) | `jagex/ObjectDefinition.java` @ 0x3de2 | `ev_fad1d07d…` |
| Models below version 13 scaled × 4 (`Model::build`) | `jagex/ObjectDefinition.java` @ 0x47dd | `ev_960e8912…` |
| Model sun strength × 768 / contrast (`Light::model_strength`) | `jagex/MeshRasterizer_Sub1.java` @ 0x2f919 | `ev_9bf231ed…` |
| Model lightness × ambient >> 7 (`Model::corners`) | `jagex/MeshRasterizer_Sub1.java` @ 0x78e8 | `ev_edd36baf…` |
| Object at (x << 9) + (size << 8) (`locations::scenery`) | `jagex/Class329_Sub1.java` @ 0xa32 | `ev_40152d8e…` |
| Type 11 drawn as type 10 with rotation + 4 (`locations::scenery`) | `jagex/GraphNode_Sub1_Sub1_Sub6.java` @ 0x1000 | `ev_14f471fc…` |
| Location x = position >> 6 & 63 (`map_locations::decode`) | `jagex/Class329_Sub1.java` @ 0x473c | `ev_5f3197a1…` |
| Location type = info byte >> 2 (`map_locations::decode`) | `jagex/Class329_Sub1.java` @ 0x47ce | `ev_52076305…` |
| Object name defaults to "null" (`ObjectDef::decode`) | `jagex/ObjectDefinition.java` @ 0x10d | `ev_90234057…` |
| Object model ids are big smarts (`ObjectDef::decode`) | `jagex/ObjectDefinition.java` @ 0x12c8 | `ev_5e3e4965…` |
| Big smart 32767 means none, −1 (`Reader::big_smart_or_none`) | `jagex/ByteBuf.java` @ 0x3a8b | `ev_44a80b60…` |
| `readSum` adds smarts while they read 32767 (`Reader::smart_sum`) | `jagex/ByteBuf.java` @ 0x38c9 | `ev_60d3c5f1…` |
| Sun towards (−50, −60, −50) (`Light::scene`) | `jagex/Atmosphere.java` @ 0x24d | `ev_c9d1959e…` |
| Camera pitch at most 2980 (`Camera::process`) | `jagex/Isaac.java` @ 0x286 | `ev_32f52491…` |
| Camera lift target (rise >> 2) × 1536 | `jagex/Isaac.java` @ 0x9a2 | `ev_cfb95c6d…` |
| Camera distance ((pitch >> 3) × 3 + zoom) << 2 (`Camera::view`) | `jagex/HitbarDefinitions.java` @ 0xf51, 0x1219 | `ev_f1cef3a8…` |
| Focal length = viewport height × i / 334 | `jagex/Class492.java` @ 0xa6c | `ev_4207ef15…` |
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
| `VARP_SMALL` = packet 115, 3 bytes (`ServerProt::VarpSmall`) | `jagex/ServerProt.java` @ 0xf8d | `ev_ca08565a…` |
| `IF_OPENTOP` = packet 37, 19 bytes (`ServerProt::IfOpentop`) | `jagex/ServerProt.java` @ 0x582 | `ev_494407cc…` |
| `IF_OPENTOP` window id: u16 +128 after two ints (`lobby::apply`) | `jagex/PacketDecoder.java` @ 0x1f09 | `ev_e20a08cd…` |
| `IF_OPENSUB` interface: u16 LE +128 at byte 12 (`lobby::apply`) | `jagex/PacketDecoder.java` @ 0x14748 | `ev_4ba8735d…` |
| `readUnsignedShort128` = (second − 128 & 0xff) + (first << 8) | `jagex/ByteBuf.java` @ 0x1264 | `ev_45408689…` |
| `IF_OPENTOP` runs the window's on-load scripts (`Ui::open_top`) | `jagex/PacketDecoder.java` @ 0x2102 | `ev_9f701330…` |
| `IF_OPENSUB` = `openSub(topUid, …, false)` (`Ui::open_sub_from_server`) | `jagex/PacketDecoder.java` @ 0x1482f | `ev_52c4e6de…` |
| World login sends the window mode, then the game area's size (`Target::display`) | `jagex/Login.java` @ 0x2982 | `ev_8c365dad…` |
| The scene's canvas is the game area (`ground::draw` viewport) | `jagex/Class507.java` @ 0xaf5 | `ev_887b9ca9…` |
| A new game area in the game sends `SCREEN_SIZE` (`Window::set_mode`) | `jagex/Class507.java` @ 0xf10 | `ev_f3db6151…` |
| `SCREEN_SIZE` = mode, width, height, anti-aliasing (`client_packets::screen_size`) | `jagex/Class388.java` @ 0x19f | `ev_f2107830…` |
| `IF_OPENSUB` type is `readUnsignedByteC` (sent negated; `server_packets::open_sub`) | `jagex/ByteBuf.java` @ 0x45a2 | `ev_633f2e2c…` |
| Entering the world turns the camera ((int)(random × 160) − 80) & 0x3fff (`camera::entry_yaw`) | `jagex/Class190.java` @ 0x14f3 | `ev_85bd48eb…` |
| `IF_OPENSUB` then runs the window's "interface opened" hooks (`Ui::run_sub_opened_hooks`) | `jagex/IdentiKitIndexLoader.java` @ 0x8c2 | `ev_a7a0084a…` |
| …through every component, containers first (`Class151.method2591`) | `jagex/Class383.java` @ 0x133f | `ev_4ff3810e…` |
| Script arrays: at most 5000 entries (`ARRAY_NEW`) | `jagex/CS2Interpreter.java` @ 0x3a284 | `ev_bcc9e335…` |
| World reply: members world after the date of birth (`WorldDetails`) | `jagex/Login.java` @ 0x5f73 | `ev_9bd5dff4…` |
| Component params: u24 key, int (or GJ string) (`Component::params`) | `jagex/IComponentDefinitions.java` @ 0x434f | `ev_f351def0…` |
| `CLIENT_SETVARC_SMALL` = packet 116, 3 bytes (`server_packets::varc`) | `jagex/ServerProt.java` @ 0xfa5 | `ev_24379ad5…` |
| …value `read128Byte` (128 − b), then id `readShortLE` (`lobby::apply`) | `jagex/PacketDecoder.java` @ 0x16872 | `ev_7b5ef3db…` |
| A varc packet sets `VARC_INT[id]` (`Ui::varc`) | `jagex/PulseEvent.java` @ 0x55f | `ev_08cba52a…` |
| `CLIENT_SETVARC_LARGE` = packet 12, 6 bytes (`server_packets::varc`) | `jagex/ServerProt.java` @ 0x200 | `ev_39b57912…` |
| `IF_ISOPEN`: 1 only when that interface is open in the component | `jagex/CS2Interpreter.java` @ 0x61187 | `ev_0f3f351a…` |
| Use-option flags = settings >> 11 & 0x7f (`instr6296`) | `jagex/IFTargetParams.java` @ 0x167 | `ev_0f478c93…` |
| `LOGOUT_LOBBY` → `logout(Class9.aBool71)` (`Ui::server_logout`) | `jagex/PacketDecoder.java` @ 0x8d5e | `ev_729054c5…` |
| `LOGOUT_FULL` → `logout(false)` | `jagex/PacketDecoder.java` @ 0x150f | `ev_7d9f5d61…` |
| `logout(true)` → game state 12 (log in to the lobby again) | `jagex/CursorIndexLoader.java` @ 0x546 | `ev_5826934d…` |
| `UPDATE_STAT`: base level = highest i + 2 whose table entry the xp reaches (`Skill::from_server`) | `jagex/PacketDecoder.java` @ 0x1353a | `ev_bdf3f9f5…` |
| Level table: running sum of (int) (l + 300 × 2^(l / 7)), / 4 (`xp_for_level`) | `jagex/Class516.java` @ 0x1a9 | `ev_588bf100…` |
| `UPDATE_STAT` = packet 140, 6 bytes (`ServerProt::UpdateStat`) | `jagex/ServerProt.java` @ 0x12db | `ev_5fa936f5…` |
| Skill maximums: 99, the 25th 120 (`SKILL_MAX_LEVEL`) | `jagex/Class516.java` @ 0x65 | `ev_863e10ea…` |
| `STAT_VISIBLE_XP` = `SKILL_XP[skill]` | `jagex/CS2Interpreter.java` @ 0x4bd99 | `ev_829cf3f9…` |
| Stat hook when the change count passed the component's (`Pass::hooks`) | `jagex/client.java` @ 0xd3d9 | `ev_7123a2dc…` |
| `VARP_SMALL` value is a signed `readByte` (`server_packets::varp`) | `jagex/PacketDecoder.java` @ 0x1132e | `ev_00abdd81…` |
| `IF_SENDTOFRONT` reorders only `method1617`'s copy, which nothing else reads | `jagex/Interface.java` @ 0x798 | `ev_c8170d68…` |
| `WORLD_QUICKCHAT` = `IS_QUICKCHAT_ONLY` (`WorldDetails::quickchat_only`) | `jagex/CS2Interpreter.java` @ 0x5b301 | `ev_daa28518…` |
| `instr6297`: option n of a component, else "" | `jagex/CS2Interpreter.java` @ 0x23d30 | `ev_512e5574…` |
| `instr6170` sets the text shadow (`"shadow"`) | `jagex/CS2Interpreter.java` @ 0x6e5de | `ev_a6b45d26…` |
| World reply: quick-chat only is its own byte (`decode_ok`) | `jagex/Login.java` @ 0x5e31 | `ev_c18cde51…` |
| `ENUM_STRING`: the enum's text, else its default (`Enum::text`) | `jagex/CS2Interpreter.java` @ 0x38b7c | `ev_f72b2ec1…` |
| A chat line goes on top, the 100th falls off (`Ui::add_chat_line`) | `jagex/ChatLine.java` @ 0x663 | `ev_37be937c…` |
| `GAME_MESSAGE` = packet 160, size in a byte (`ServerProt::GameMessage`) | `jagex/ServerProt.java` @ 0x15aa | `ev_fcb0ec93…` |
| Line n exists only for 0 ≤ n < 100 (the chat getters) | `jagex/ChatLine.java` @ 0xb69 | `ev_1dd754af…` |
| A line's number = `++anInt801 − 1` (`Ui::add_chat_line`) | `jagex/MaterialProp25.java` @ 0x140 | `ev_e7eb1a91…` |
| A line's time = `CYCLES_20MS` (`Ui::add_chat_line`) | `jagex/ChatLine.java` @ 0x27b | `ev_c369267b…` |
| 100 chat lines kept (`CHAT_LINES`) | `jagex/ChatLine.java` @ 0x92 | `ev_cfbaa526…` |
| Each line: `NUM_CHAT_LINES` + 1, `anInt7391` = this cycle (`Ui::add_chat_line`) | `jagex/ChatLine.java` @ 0x7d5 | `ev_83c263e8…` |
| `GAME_MESSAGE` type 99 → the console, not the chat (`game_message`) | `jagex/PacketDecoder.java` @ 0x5f57 | `ev_e91b439a…` |
| `GAME_MESSAGE` → `appendChatMessage(type, effects, simple, formatted, simple, message)` | `jagex/PacketDecoder.java` @ 0x60de | `ev_a57ac1f2…` |
| Chat hook when `anInt7391 > anInt1442` (`Pass::hooks`) | `jagex/client.java` @ 0xdbac | `ev_101d1fc4…` |
| `RUN_ENERGY` = packet 64, 1 byte (`ServerProt::RunEnergy`) | `jagex/ServerProt.java` @ 0x8ed | `ev_ec7ed77d…` |
| …`RUN_ENERGY = readUnsignedByte`, `anInt7397` = this cycle (`Ui::set_run_energy`) | `jagex/PacketDecoder.java` @ 0x3250 | `ev_59175a0f…` |
| Run-energy hook when `anInt7397 > anInt1442` (`Pass::hooks`) | `jagex/client.java` @ 0xe28a | `ev_1fa1b45d…` |
| `LOGOUT_FULL` = packet 62, no body (`ServerProt::LogoutFull`) | `jagex/ServerProt.java` @ 0x8bd | `ev_4aea9db0…` |
| `LOGOUT_LOBBY` = packet 9, no body (`ServerProt::LogoutLobby`) | `jagex/ServerProt.java` @ 0x169 | `ev_651c3841…` |
| `aBool71` set when the lobby window opens (`Stage::World::from_lobby`) | `jagex/QuickChatMessage.java` @ 0x26b3 | `ev_5ba23c88…` |
| Window mode: 3 full screen, else 2 resizable, else 1 (`Window::display`) | `jagex/Class158.java` @ 0x8b | `ev_14321347…` |
| In the game = game states 12, 13, 18; only then `SCREEN_SIZE` (`Ui::in_game`) | `jagex/GameState.java` @ 0x161 | `ev_53a20b49…` |
| A varbit = var >> start & mask (`Ui::varbit`) | `jagex/PlayerVarProvider.java` @ 0xff0 | `ev_0f5e506c…` |
| Each variable change goes into a 32-entry ring (`VarpChanges::record`) | `jagex/PulseEvent.java` @ 0x417 | `ev_3f05f340…` |
| `FRIEND_COUNT` is −2 while the list isn't loaded (`Social::friend_count`) | `jagex/CS2Interpreter.java` @ 0x34d71 | `ev_b1ad047f…` |
| Membership end in minutes = `MEMBERSHIP_END / 60000` (`USERDETAIL_LOBBY_MEMBERSHIP`) | `jagex/CS2Interpreter.java` @ 0x45bda | `ev_a8f72ecb…` |
| `aLong420` = end − now − 5-byte value (time left counts down from login) | `jagex/Login.java` @ 0x65ca | `ev_f458cc8b…` |
| Varbit opcode 1 = base variable u16, start, end (`cache/varbit.rs`) | `jagex/VarBitDefinitions.java` @ 0x2cf | `ev_e0614b36…` |
| Option n allowed when settings bit n + 1 is set (`menu::enabled`) | `jagex/IFTargetParams.java` @ 0x222 | `ev_3ee13d03…` |
| `IF_OP1` = client packet 96, 8 bytes (`ClientProt::IF_OPS`) | `jagex/ClientProt.java` @ 0xc1c | `ev_09bafb3a…` |
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
There is no reference server, so `src/server/login/accounts.rs` decides: the first login with a name
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
region from (chunk − 6) / 8 to (chunk + 6) / 8). The server sends it (`src/protocol/region.rs`, `src/server/world/enter.rs`,
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
(`src/server/lobby/mod.rs`; the value is this server's own: the script only tests ≠ 0).
Packets (`src/protocol/server_prot.rs`, from `ServerProt.java`, 162 ids with sizes; all listed in `docs/PROTOCOL.md`): the
id as an ISAAC-scrambled smart, a u8 length for size −1, a u16 for −2, then the payload.
`VARP_SMALL` (115, size 3): value `readByte`, id `readUnsignedShortLE128` (low byte − 128, then
high). `VARP_LARGE` (8, size 6): value `readIntV2` (bytes 2, 1, 4, 3), id LE+128. The client
reads whatever has arrived each tick without blocking, keeps a two-byte id's first half when a
read splits it, and stores variables in `Ui::varp` for `LOAD_VARP`. Ported for the lobby's
scripts: `LOAD_VARP`, `CC_SETCHILD`, `CC_SETVFLIP`, `CC_SETTRANS`, `IF_SETTEXTFONT`, `instr6232`
(release hook), `instr6222` (option cursor: popped, no custom cursors drawn yet).

### The game's window (`IF_OPENTOP`, `IF_OPENSUB`)
A world login keeps its connection open after `REGION` (`login.rs` hands it to
`client/lobby.rs`, the same reader as the lobby's), and on entering the world the login or lobby
window closes. `IF_OPENTOP` (37, 19 bytes: key `readIntV1`, flags `readIntV1`, window
`readUnsignedShort128`, type `readUnsigned128Byte`, two ints) makes the window the root
(`client.BASE_WINDOW_ID`, `ClipFlagMap.method6007`), lays it out (`Class516.method8867`) and
runs its on-load scripts (`Class150.method2582`); interfaces already open stay. `IF_OPENSUB`
(38, 23 bytes: key int, flags `readIntV2`, an int V2, interface `readUnsignedShortLE128`, an int
LE, parent component int, type `readUnsignedByteC`) opens the interface in that component and
runs its on-load scripts (`IdentiKitIndexLoader.openSub` with false). The XTEA keys these
carry, and `IF_OPENTOP`'s type 2 (`ParamIndexLoader.method7081`), aren't used yet.
Which window is the server's choice: this server sends 746 for the resizable modes and 548 for
the fixed one (`login/world.rs` `game_window`), as 727 servers do; it opens nothing in it yet.
The world login carries the real mode and game area (`Class158.windowedMode`: 3 full screen, 2
resizable, else 1; then `BASE_WINDOW_WIDTH`, `BASE_WINDOW_HEIGHT`; `Window::display`). Each time
the game area is set again in the game (`Class507.method8725`, `GameState.loggedIn`), the client
sends `SCREEN_SIZE` (84, 6 bytes: mode, width u16, height u16, the anti-aliasing preference;
`Class388.method6692`), and this server opens the other window when the mode needs it.
The side panels (`login/world.rs` `side_panels`): the tab buttons are 548:112-119 (Combat
Styles, Task System, Stats, Quest Journals, Inventory, Worn Equipment, Prayer List, Magic
Spellbook) and 548:83-90 (Resources, Friends List, Friends Chat, Clan Chat, Options, Emotes,
Music Player, Notes); each one's click hook is script 1303 with its tab, 0-15 in that order.
Script 1303 stores it (varc 168) and through 1304 calls 1387, whose switches show 746:(112 +
tab) in the resizable window and 548:(176 + tab) in the fixed one, hiding the others; 548:180
(tab 4, the inventory) is the one not hidden in the cache. These are read from the cache's
interfaces and scripts. Which interface goes in each slot is the server's: this server opens,
with `IF_OPENSUB` type 1 after `IF_OPENTOP`, interfaces recognised by their text in the cache:
884 combat styles, 1056 task system, 320 stats, 190 quests, 679 inventory, 387 worn equipment,
271 prayer, 192 magic, 550 friends, 1109 friends chat, 1110 clan chat, 261 options, 590 emotes,
187 music, 34 notes; Resources (tab 8) stays empty.
The game window's scripts (`client/cs2/ops.rs`), ported as the reference runs them so the
window's and the panels' on-load scripts go further: hook setters for game events, kept until
those events are ported (`instr6257` inventories, `instr6507` stats, `instr6393`/`instr6899`,
`instr6450`, `instr6103`/`instr6376`, `instr6094`, `instr6095` (variables, as `instr6237`),
`IF_SETONMOUSELEAVE`; their trigger lists are dropped), and `instr6253`/`instr6111`
(`anObjectArray1346`), which `IF_OPENSUB` fires (`Class383.method6514(BASE_WINDOW_ID, 1)` →
`Class151.method2591`: every component of the window, a container's created children and the
interface open in it first). `instr6805`/`IF_ISOPEN`/`instr6295` (what is open in a component),
`ARRAY_NEW` (operand: array << 16 | type, 'i' fills with 0, others −1; at most 5000) /
`ARRAY_LOAD` / `ARRAY_STORE` (five arrays per executor; kept on the `Ui` for the depth hooks run
at), `INV_SIZE` (inventory definition opcode 2, config archive 5), `INV_GETITEM` (−1: no
containers yet), `STAT`/`STAT_BASE` (0 until stats arrive), `WORLD_MEMBERS` and `USERDETAIL_DOB`
(from the login replies), `COMLEVEL_ACTIVE` (0 until the player's appearance),
`GET_SYSTEM_UPDATE_TIMER` (0), `GET_MOUSEBUTTONS`, `instr6681`/`instr6456` (camera yaw and
pitch >> 3), `instr6141` (the view component's size: `Class492.method8265` with the client's
limits 1 and 32767 never letterboxes), `instr6289` (font), `instr6296` (settings >> 11 & 0x7f),
`IF_SETTILING`, `CC_SETGRAPHICSHADOW`, `LOWERCASE`, `BIT_AND`, component params
(`CC_SETPARAM_INT`/`instr6191` set, or remove when equal to the param's default; `instr6771`
reads, else the default), and `IF_SETALPHA`/`IF_SETCLICKMASK`/`CC_SETCLICKMASK` (popped: flags
this renderer and input don't use yet). Still stopping the window's scripts: `IF_SENDTOFRONT`
(draw order), `ENUM_STRING`, `CC_RESETOPTIONNAMES`, `instr6527`, `instr6297`, and the item ones
(`ITEM_PARAM`, `IF_SETITEM`) that need item definitions.
Client variables from the server: `CLIENT_SETVARC_SMALL` (116: value `read128Byte`, id
`readShortLE`) and `CLIENT_SETVARC_LARGE` (12: id `readShortLE`, value `readIntV2`) set
`VARC_INT[id]` (`PulseEvent.method6751`, type 1; the varc change triggers aren't ported). Varc
1413: every script that reads it returns while it is 0 (1387, which shows a tab's slot, 1534,
2688) and no script sets it, so it is the server's to set; this server sends 1413 = 1 right
before `IF_OPENTOP`, because the window's on-load scripts pick the first tab (varc 168 = 0,
Combat Styles) and run 1387 then. With it, the tab buttons switch panels (script 1303 → 1304 →
1387; clicking the open tab closes it, varc 168 = −1).
Not in the reference, by request: in the resizable mode the game area is never smaller than
765 × 503 (`resizable_area`), so a small window cuts the game's window off at its right and
bottom edges (the area then starts at the window's left) instead of piling its parts onto each
other.
The orbs by the minimap: the 746 slots carry their tooltips in the cache (746:196 "Your current
Life Points.", 197 Prayer Points, 198 run energy, 199 Summoning energy, through script 4102);
the 548 slots (160, 161, 162, 164, 57 × 34 like the orb interfaces) are in the same order, and
script 2756 pairs 548:162 with 746:198. This server opens 748 (life, "Use Cure"), 749 (prayer),
750 (run) and 747 (summoning) there with `IF_OPENSUB` (`login/world.rs` `ORBS`). Skills: `UPDATE_STAT` (140, 6 bytes: experience
`readInt`, skill and level `readUnsignedByte`) sets `SKILL_XP`, `SKILL_LEVEL_VISIBLE` (`STAT`)
and `SKILL_LEVEL_ACTUAL` (`STAT_BASE`): 1, or i + 2 for the highest i below the skill's maximum
(99, 120 for the last of the 25) whose `SKILL_XP_FOR_LEVEL[i]` the experience reaches; the
table is the running sum, for l = 1..120, of (int) (l + 300 × 2^(l / 7)), divided by 4 (83 for
level 2, 1154 for 10, 13 034 431 for 99). Each one notes the skill in a 32-entry ring
(`anIntArray7283`, `anInt7384`); in the interface pass a component's `anObjectArray1387` hook
(set by `instr6393`/`instr6899`, its skills in `anIntArray1402`, from the cache or the hook)
runs as the variable hooks do (`client.java:1570`). The orbs: life is varbit 7198 (varp 1240,
bits 1-15) out of max(Constitution base, 1) × 10 (script 2916), prayer varbit 9816 (varp 2382,
bits 0-14) out of max(Prayer base, 1) × 10 (script 5255), summoning the skill itself. This
server sends a new player every skill at 1 but Constitution at 10 (1154 experience), and full
life (100) and prayer (10) points, before the window (its choices). `VARP_SMALL` reads a signed
byte, so this server sends values outside −128..=127 with `VARP_LARGE`. Run energy: `RUN_ENERGY` (64, one
unsigned byte) sets `client.RUN_ENERGY` (`RUNENERGY_VISIBLE`) and stamps `anInt7397` with the
client cycle `anInt7347` (counted up after each interface pass, `pulseSubInterfaces`); in the
pass a component's `anObjectArray1419` hook (set by `instr6251`) runs when that stamp is newer
than its `anInt1442`, which then becomes the cycle (`client.java:1662`; the other game-event
stamps there aren't ported yet). This server sends 100 (full; its choice) before the window. The chat (`login/world.rs` `CHAT`): the frame 752 goes in the chat area, 548:168 and 746:22,
which script 84 handles together (`IF_SETNOCLICKTHROUGH` on both); the chat box 137 in 752:9,
which script 1364 (the chat area's layout, run through the "interface opened" hooks) shows
when nothing is open in 752:10-13; the filter bar 751 (519 × 23) in 548:53, the 519 × 23 slot
of the bottom bar, and in 746:23, the only 519-wide slot along 746's bottom (inferred from its
shape and place). 751 opens first: script 178 resizes it to its own `IF_GETWIDTH`, 0 until it
is open and laid out (scripts reach components of interfaces that aren't open,
`IComponentDefinitions.getDefs` loading them, so it can run before). Ported for those scripts:
`instr6170` (text shadow), `CC_SETALPHA`, `CC_RESETOPTIONNAMES`, `CC_GETTRANSPARENCY`,
`instr6527` (`anObjectArray1396`), `instr6132` (active text), `instr6297` (option n, else ""),
`instr6225` (`ChatLine.NUM_CHAT_LINES`, 0), `CHAT_GETFILTER_PUBLIC`/`_TRADE` (0 until the server
sends filters), `CHAT_PLAYERNAME` ("" until the player's appearance), `WORLD_QUICKCHAT` (the
world reply's sixth byte), `ENUM_STRING`, and `IF_SENDTOFRONT`/`IF_SENDTOBACK`: on a cache
component they reorder `Interface.iComponentDefinitions` (`method1617`), which nothing else
reads (drawing and input walk `components`), so they have no visible effect.
Chat messages: `GAME_MESSAGE` (160, its size in a byte): type `readSmart`, effects `readInt`,
flags; with flag 1 the sender's name, with flag 2 also its formatted form (else the same), then
the message. Types 99 and 98 go to the developer console (not ported); the rest go through
`ChatLine.appendChatMessage(type, effects, simple name, formatted name, simple name, message)`
(the ignore list is empty): the line goes on top of the last 100 (`CHAT_LINES`, newest first),
gets a running number (`MaterialProp25.method15396`) and the clock (`CYCLES_20MS`),
`NUM_CHAT_LINES` counts up (`instr6225`) and `anInt7391` stamps the cycle, so in the interface
pass the `anObjectArray1413` hooks run (`client.java:1620`, with the other game-event stamps on
`anInt1442`). The chat box's scripts (81 → 1560 → 84, which lays out each shown line) read a
line n, counting from the newest, through `instr6739` (type), `instr6565` (quick-chat
message), `CHATLINE_GETCYCLES20MS`, `instr6682` (number) — −1 for a missing line — `instr6572`
(effects, 0), and `instr6189` (crowned name), `instr6560` (message), `instr6573` (name),
`instr6577` (plain name), `CHAT_GETCLAN` ("" when missing); `instr6148` is the active
component's font. This server sends "Welcome to Open 727." (type 0, its own words) after the
window.
Logging out of the game: the Exit button (548:159 runs script 115 with 99, 746:195 script 1303
with 99) opens tab 99, which script 1387 shows in 548:194 or 746:130. This server opens
interface 182 there; its on-load script 1104 names 182:6 "Exit to Lobby" and 182:13 "Exit to
Login", both `IF_OP1` to the server. The server answers `LOGOUT_LOBBY` (9) or `LOGOUT_FULL`
(62), no body, and ends the session (the server's part). The client: `LOGOUT_FULL` →
`CursorIndexLoader.logout(false)`: the connection closes and the login screen opens (game
state 5), the name and password forgotten; `LOGOUT_LOBBY` → `logout(Class9.aBool71)`
(`aBool71` set once the lobby was shown): game state 12, where `Login.method5018` logs in to
the lobby again with the saved name and password; without the lobby, as `LOGOUT_FULL`. The
packets that arrived before the server closed the connection are applied first.
The scene is drawn only in the game area, as the reference renders into the game canvas
(`gameCanvas.setSize(BASE_WINDOW_WIDTH, BASE_WINDOW_HEIGHT)` at `GAME_CANVAS_X`, 0): fixed
765 × 503, or the window's size capped by the max screen size preference, centred across the
top, black around it; the camera's view uses the area's height.
The reference draws the scene and the interfaces into one buffer, the scene first. Here the
scene is drawn by the GPU, so the interfaces are drawn into a raster that starts see-through
(`Raster::clear_see_through`: the top byte of each pixel is how much of the scene still shows,
shrinking as colour is laid over it) and laid over the scene. Approximation: the scene fills the
game area, not only the window's view component.

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
lobby reply (`src/server/lobby/mod.rs`; both values are this server's own). A hook set after the
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
Client packets (`src/protocol/client_prot.rs`, from `ClientProt.java`, 104 ids): the id
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
step `lobby-bar name` opens the lobby with it. The game's windows have the same bar, 50 high:
548:2 (fixed, with 548:204 under it at y 50) and 746:239 (resizable, with 746:48 under it at
y 50, height "parent − 50"); the same setting hides it the same way when `IF_OPENTOP` opens
either (`Ui::open_top`).

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

### The ground (first step: `client/ground.rs`)
The scene is 104 × 104 tiles (`MapSize.SIZE_104`) around `REGION`'s centre chunk (cx, cy):
regions (cx ± 6) / 8 by (cy ± 6) / 8, base tile ((cx − 6) × 8, (cy − 6) × 8)
(`MapRegion.loadMapSceneNormal`, `coordGrid`). Underlay colours are split once per definition
(`UnderlayDef.method8048`: hue × chroma, saturation, lightness, chroma ≥ 1), summed over the
11 × 11 tiles around each tile (`Class329`, running sums over x then y), and packed as
`Class371.method6348(hue × 256 / chroma, saturation / count, lightness / count)`; an overlay with a
colour covers it. Plane 0 is drawn in 3D (tile corners at `tileHeights`, 512 units a tile,
negative up), east drawn to the right, through the reference's camera (below). The self-test
saves `world.png`.

Shapes (`Class329.method5845`, the path with ground blending off): a tile's shape (shape 0 with no
overlay becomes 12, all underlay) gives `OVERLAY_FACE_COUNT` overlay and `UNDERLAY_FACE_COUNT`
underlay triangles, overlay first, each three of 13 points (`anIntArray3811` / `3800`, picked by
`anIntArrayArray3824` / `3860` / `3815`), turned by the rotation: 1 (y, 512 − x), 2 (512 − x,
512 − y), 3 (512 − y, x). An overlay with neither colour (`primaryRGB` and `secondaryRGB` −1) is
left out but its shape still cuts the underlay; a face with colour −1 isn't drawn; underlay faces
take the blended colour (0 where the blend wrote nothing). Points sit at `Ground.averageHeight`.

Light (`OpenGLGround`, the toolkit the client runs): a normal at each corner inside the scene,
(dx, −1024, dy) made length 1 from the corner heights either side (0 on the scene's edges), blended
across the tile for points between corners. Each point's colour is its lightness × (74 − shadow)
/ 128 (2..=126) through the colour table, then each channel × ambient + d × (0.69921875 if d > 0
else 1.2), with d the normal · the sun, clamped. The scene light (`Class239.method4049` with
`Atmosphere`'s defaults): ambient (0.7 + brightness preference × 0.1 + a random −0.05..0.05 per map
load, `MapRegion.aFloat3173`) × 1.1523438; sun white towards (−50, −60, −50) × 4. With light detail
off `SA` works this out itself; with it on OpenGL's two lights (the sun, and −1.2 from the other
side) give the same. Not yet: textures, scenery shadows (counted as 0), the planes above and
locations.

Ground blending (`method5846`, used when the preference is on, the default; `client/tiles.rs`):
- A whole-underlay tile (shape 0 or 12) inside the scene picks its diagonal (`method5870`): each
  corner scores +1/−1 for the 3 tiles touching it having the same underlay id (compared as signed
  bytes), and rotation 1 is used when |sw − ne| < |se − nw| (on a tie, the flatter diagonal).
- Overlays with a colour around the tile (`method5848`) claim its 8 edge points: a blending
  overlay (opcode 12) on a diagonal tile takes the touching corner; on a side tile it takes the
  3 shared points it covers when no higher priority has them (priority (opcode 11, default 8)
  << 8 | id, `OverlayDef.method11356`); the tile's own blending overlay does the same for its
  points. Any side neighbour marks an edge on that side when this tile's shape leaves it open; so
  do sides whose two corner points ended with different overlays (`method5859`). A shape 0 / 12
  tile with two edges meeting at a corner becomes shape 13 / 14 turned to that corner.
- The cut (`method5849`): no edges and no blending overlay over an underlay → the plain tables;
  a blending overlay over an underlay → `anIntArrayArray3775`...; else `anIntArrayArray3774`...
  Triangles at a side with an edge are split at the side's middle point.
- Overlay points take a higher-priority neighbour's colour at claimed edge points (`method5850`);
  underlay points take the claiming overlay's colour, the blending overlay's where it covers,
  the blended colour of the tile at that corner, or between corners `Class371.method6347` of the
  four (lightness, saturation and hue mixed apart, by x then y) (`method5851`).

### Map locations (`l{x}_{y}`; `client/locations.rs`, `cache/map_locations.rs`, `cache/object_def.rs`)
`REGION` carries one XTEA key (4 ints) per region around the centre chunk, in `ground::regions`'
order; the client keeps them (`Arrival::keys`). Each region's `l{x}_{y}` (index 5, by name hash)
file 0 is decrypted with its key (`Index.getFile(archive, 0, key)`, `MapRegion`). The file
(`Class329_Sub1.method12461`): object ids as running sums of `readSum` (smarts added while 32767)
from −1, 0 ends; for each, positions as running sums of `readSmart` − 1 (0 ends): plane =
pos >> 12, x = pos >> 6 & 63, y = pos & 63; then a byte type << 2 | rotation. A location is kept
when its scene tile is strictly inside (1..103 both ways). Object definitions: index 16, archive
id >>> 8, file id & 0xff (`SharedConfigsType.OBJECTS(6, 8)`), decoded by `method7965` (opcode 1
model sets per type with big smarts, 32767 = none; 2 name, default "null"; any opcode it doesn't
know is skipped without reading). `--map-locations <region>` lists a region's locations. The
self-test counts them around the spawn (10577, nearest named: bushes and a door).

### Object models (`client/object_model.rs`, `client/locations.rs::scenery`)
Placement (`Class329_Sub1.method12459`, every plane): size swapped for rotations 1 and 3; the
model sits at ((x << 9) + (sizeX << 8), (y << 9) + (sizeY << 8)) at the average of the four
corner heights around the middle of the tiles it covers. Scenery (10), roofs (12..=21) and ground
decoration (22; plain decoration only when the `groundDecor` preference is on) are drawn; type 11
is type 10 with rotation + 4 (`GraphNode_Sub1_Sub1_Sub6.method16117`); `hidden` objects only with
textures on. The model (`ObjectDefinition.method7971`): the models listed for the type, each × 4
when its version (new format flag 8, byte 24 from the end) is below 13, merged when several
(`RSMesh(RSMesh[], n)`: corners at the same place share a vertex); mirrored on z for `inverted` or
a whole-corner wall past rotation 3; type 4 past 3 turned 2048 and moved (180, 0, −180); turned
4096 × (rotation & 3) (positions and normals); recoloured (a palette byte gives 0); scaled
(× s / 128) and moved by the offsets; scenery past rotation 3 turned another 2048, positions only
(`method8010`). Ground fitting (`pa`): mode 1 adds the ground's height under each vertex minus the
model's; mode 2 does so scaled by (param − y × 65536 / min y) / param where that is positive; both
only if the model lies within the scene and not all four corner heights under it equal its own.
Mode 4 adds the ground above's height minus the model's, plus the model's height span (max y −
min y); mode 5 sets y = ((below − above − param) × ((y << 8) / span) >> 8) − (model height −
below); both need the plane above (none on plane 3) and the model within the scene.
Then the 164..=166 nudge. Normals (`MeshRasterizer_Sub1`): each face's cross product halved to
within ±8192 and scaled to 256; a smooth (type 0) face adds it to its three vertices, a flat
(type 1) face keeps its own with count 0; type 2 faces aren't drawn. Colour: lightness × (64 +
ambient) >> 7 (2..=126) through the colour table; each channel × scene ambient + d × (0.69921875
if d ≥ 0 else 1.2) × 768 / (850 + contrast), d = sun · normal / (count × 256), or / 384 at count
0; alpha 255 − face alpha. Not yet: contouring 3, animations.

Textured faces on models (`object_model.rs`, `MeshRasterizer_Sub1`'s constructor and
`method13531`). A face's texture is its mesh texture, except a ground texture
(`isGroundMesh`) while textures are off (`Class94.method1589` sets flag 0x40 then, and the
constructor drops those). Texture coordinates per corner: no mapping (−1) gives (0, 1), (1, 1),
(0, 0); a kind 0 mapping through vertices p, m, n gives, in floats, e1 = m − p, e2 = n − p,
normal = e1 × e2, u = ((e2 × normal) · (corner − p)) / ((e2 × normal) · e1), v = ((e1 ×
normal) · (corner − p)) / ((e1 × normal) · e2); mirroring swaps corners 0 and 2's coordinates
with them. Colour (`method13531`): the face's colour as before, then for a texture with a shadow
factor s (of 256) it moves towards the grey of the model's ambient (64 + ambient: 0 below 0,
white above 127, else × 131586): ((grey & 0xff00ff)·s + (c & 0xff00ff)·(256 − s) & 0xff00ff00
+ (s·(grey & 0xff00) + (256 − s)·(c & 0xff00) & 0xff0000)) >> 8; then a brightness b brightens
each channel × (b + 256) (at most 65535) >> 8. Lighting as before. Each texture the scene uses is
made once (`texture::render`: 64 × 64 when half size, else 128 × 128, brick tiles column by
column) and drawn multiplied by the corner colours, its coordinates wrapping (the reference's
textures repeat; here a shader takes `fract(uv)`, as macroquad has no safe repeat setting). The
textures come from the cache through `Assets` (index 26's table, index 9's materials by
`Index.getFile(id)`, index 8's sprites). Around the spawn 232 textures are drawn.

Drawn faces (`MeshRasterizer_Sub1`'s constructor): not type 2, and not a face whose texture is
marked `skipTriangles` (unless it's a ground texture dropped with textures off).

Projected mappings (kinds 1..=3; `MeshRasterizer.method11256`, `method11257`, `method11306`,
`method11255`, `method11271`, `method11254`). For these kinds the mapping's three numbers are a
direction (signed), not vertices. Each such mapping's centre is the middle of the box around the
vertices of the drawn faces using it ((min + max) / 2 per axis, ints). Its matrix: a turn about y
by `particleLifespanX & 0xff` × 0.024543693 (cos, sin as floats: [c 0 s; 0 1 0; −s 0 c]),
times the tilt onto the direction (y = dirY / 32767, sy = −√(1 − y²), k = 1 − y, flat =
√(dirZ² + dirX²); unless flat and y are both 0: a = −dirZ / flat and b = dirX / flat when flat
isn't 0, else a = 1, b = 0; tilt [y + a²k, b·sy, b·a·k; −b·sy, y, a·sy; a·b·k, −a·sy, y + b²k]),
each row then scaled: kind 1 by (1, 64 / scaleY, scaleX / 1024) for a positive scaleX,
(−scaleX / 1024, 64 / scaleY, 1) for a negative one, (1, 64 / scaleY, 1) for 0; kind 2 by 64 /
each scale; kind 3 by each scale / 1024. A corner (x, y, z) = matrix · (vertex − centre), in
int × float. Turn code t = `particleLifespanY`, shift = `particleLifespanZ` / 256. Kind 1
(cylinder, `method11306`): u = atan2(x, z) / 2π + 0.5 (× scaleZ / 1024 unless that is 1), v =
y + 0.5 + shift. Kind 2 (cube, `method11255`): the face's normal (ints from its corners) through
the matrix, each part over 64 / that scale, picks the side (`method11254`: |y| largest → 0 if y >
0 else 1; |z| → 2 or 3; else x → 4 or 5); with p = colours[0] / 256 and q = colours[1] / 256:
side 0 (x + shift, −z + q), 1 (x + shift, z + q), 2 (−x + shift, −y + p), 3 (x + shift, −y +
p), 4 (z + q, −y + p), 5 (−z + q, −y + p), each + 0.5. Kind 3 (sphere, `method11271`): u =
atan2(x, z) / 2π + 0.5, v = asin(y / |(x, y, z)|) / π + 0.5 + shift. Then (u, v) is turned: t 1
(−v, u), 2 (−u, −v), 3 (v, −u). Kinds 1 and 3 then mend the seam: along u (v when t is odd) a
corner more than half the wrap (scaleZ / 1024 for kind 1, 1 for kind 3) from corner 0 moves back
by the wrap. Other kinds give all 0.

Ground textures (`tiles.rs`, `ground.rs`; `Class329.method5845`/`5850`/`5851`, `OpenGLGround.method6707`
and `SA`, `Node_Sub3`). Floors carry a texture and a scale: an overlay's `texture` and `anInt7057`
(opcode 9, u16 << 2, default 512), an underlay's `texture` and `scale` (opcode 3, u16 << 2,
default 512). Every point of a tile's triangles gets a paint (colour, texture, scale): without
blending (`method5845`) the overlay's or the underlay's; with blending, an overlay point that
another overlay outranks takes that overlay's (as its colour), an underlay point takes an
overlay around's (priority ≥ 0), the blending overlay's where it covers (with no overlay, texture
−1 and scale 256), else at a corner the underlay of the tile at that corner (this tile's where
that one has none) and inside the tile the underlay of the quarter it is in, the colour blended
as before. A point's material is its texture and scale, none where it has no colour, and a ground
texture (`isGroundMesh`) with textures off becomes no texture at scale 128 (the ground's flag
0x20, set when `textures` is off, `MapRegion`). Materials draw in the order of their key (a long
of water values, scale << 14 | texture; no texture is −1 and comes first; sorted by
`Class214.method3669`): each triangle is drawn once per material among its points, the first
opaque wherever a point has a material, each later one opaque only at its own points
(`method12081`) and clear at the others, so the textures fade into each other. A material's
colour at a point (`method12083`): the point's lit colour (lightness × 74 >> 7 through the colour
table), then for a texture with a shadow factor (unless its `effectId` is 4) moved towards the
grey of 74 (× 131586) and brightened by its brightness, as on models, then × the light. Texture
coordinates are the point's fine position over the scale (`glScalef(1 / scale)` on the texture
matrix). The whole ground is drawn before the objects.

Sliding textures (`OpenGLRenderer.method13609`, `method13602`; `scene.rs` `slide`): a texture
with `textureSpeedU` or `textureSpeedV` (signed bytes in the texture table) has its coordinates
moved by ((t mod 128000) / 1000) × speed / 64 mod 1 (floats, Java's remainder keeping the
sign), t the clock `client` hands the renderer each frame (`method8398((int) Utils.time())`:
milliseconds since 1970 cut to an int). On the ground the scale divides first, then the slide
is added (`Node_Sub3`: `glScalef` after the renderer's `glTranslatef`). Around the spawn one of
the drawn textures slides. Here the draw order across the scene is
by texture id (no texture first) rather than by scale then texture, and the water values (high
detail water) aren't part of the key yet.

Model textures (`cache/mesh.rs`, read, not drawn yet). New format (`decodeNewFormat`): the file's
first `texturedFaceCount` bytes are the mappings' kinds (`textureRenderTypes`); with flag
`hasFaceTextures`, after the face index data each face's texture is a u16 − 1 (−1 none), and when
the model also has mappings each textured face then has its mapping, a byte − 1 (−1: its own
three corners), from the next section (`textureIndices` bytes long). After the z section come the
mappings (`decodeTexturedTriangles`), each from its kind's sections in turn: kind 0 three vertex
numbers (u16); kinds 1..=3 three vertex numbers from the next section, a scale (x u16, y u16 —
u24 from version 14 —, z u16; all three u24 from version 15) from the next, one byte from each of
the next two and one from the last, where a kind 2 mapping then reads two colour bytes. Old
format (`decodeOldFormat`): a face whose render type has bit 2 is textured: its colour field is
its texture (65535 → −1), its colour becomes 127 and its type >> 2 its mapping; the mappings are
all kind 0, three u16 each after the colours; a face whose mapping is its own three corners loses
it, and with none kept (or no texture but −1) the lists are dropped. `upscale` also × 4 the
first as many mappings' scales as there are projected mappings (z not for kind 1). `RSMesh(RSMesh[],
n)` keeps textures (−1 for parts without) and mappings (each part's mapping numbers moved past the
earlier parts' mappings; kind 0 vertices found again like the faces' after all faces, the other
kinds keep their part's vertex numbers, as the reference does).

Planes: all four are built (`Class329.method5837` loops the planes); each plane's ground uses its
own heights, floors and normals, but the blended underlay colours are one table for all planes
(`ints_5`), so a plane keeps the colour below where it blends nothing. Each location sits on its
own plane's ground (`method12461` passes the plane twice). The camera's heights are the player's
plane (`Isaac.processCamera`).

### Textures (first step: `cache/material.rs`)
The texture table is index 26, file 0 (`IndexLoaders.TEXTURE_DEFINITION_INDEX`, read by
`ImageIndexLoader`'s constructor): a u16 count, a byte per texture (1 = defined), then each field
for every defined texture in turn: ground mesh (byte, 0 = yes), half size, skip triangles,
brightness, shadow factor, effect, effect parameter, colour (u16, packed HSL), speed u, speed v,
a flag, brick tile, mipmaps, repeat s, repeat t, HDR, combine mode, effect parameter 2 (int),
blend type. Each texture's material is index 9 (`Index.getFile(id)`: archive 0's file id when the
index has one archive). `MaterialDefinitions(ByteBuf)`: an operation count; each operation is a
byte, its kind (`MaterialProperty.getById`, 0..=39), its palette size, a settings count and that
many (setting, fields) pairs read by the kind's own `decode` (operation 34's setting 2 lists
`anInt9736` shorts when negative, operation 29's shapes are 12, 20, 15 or 15 bytes by type), then
one byte per input (the kind's `numParams`); then the opaque, translucent and combined outputs.
`--texture-stats` reads them all: 2315 textures, 1484 ground meshes, every material decodes;
operations by how many textures use them: 0 in 2176, 8 in 1888, 7 in 1573, 34 in 1081, 10 in
873, 5 in 827, 32 in 760, 21 in 749, 6 in 744, 30 in 723, 36 in 676, 15 in 620, 3 in 617, 1 in
597, 39 in 575, ... (nearly all 40 are used).

Pixels (`client/texture.rs`, `MaterialDefinitions.renderIntPixels`): every operation gives a row
of one channel (`method12319`, `noPalette`) or three (`getPixels`), values 0..4096, pulling its
inputs' rows as needed (`method12317` takes red of a coloured input, `method12333` repeats a
single channel). x and y are (i << 12) / size (`Class316.method5593`); rows and columns wrap. The
output: for each row, the opaque output's colour and the translucent output's first channel,
written right to left (x from size − 1 down; a brick tile column by column); each channel >> 4
clamped 0..255 through the brightness table (int)(pow(i / 255, 0.7) × 255), at most 255
(`LRUCache.method3895`); alpha the translucent value >> 4 clamped, 0 where the colour is black.
Ported operations: 0 constant ((byte << 12) / 255, default 4096); 1 colour (24-bit, each channel
× 16); 2 / 3 x / y; 5 box blur (radius x, y; running sums × 65536 / (2r + 1) >> 16); 6 clamp
(low, high); 7 blend of two inputs by mode (1 add, 2 subtract, 3 multiply, 4 divide, 5 screen, 6
overlay, 7 dodge, 8 burn, 9 min, 10 max, 11 difference, 12 exclusion, others 0; default 6); 8
curve (257 entries; linear, 1 cosine through `Class316.anIntArray3675`, 2 cubic with extended
ends; clamped to ±32767); 10 colour ramp (257 entries; presets 1..=6); 9 mirror (x, y, both on
by default); 17 HSL shift (`method15400` to hue/saturation/lightness, add the shifts (hue a
short, the others byte × 4096 / 100), wrap hue into 0..4096, clamp the rest, `method15401` back;
a hue of exactly 4096 matches no case and keeps the previous pixel's colour; for lightness 0 the
reference's blue is l × −680551435, a deobfuscation leftover kept as it is); 19 warp (read input
0 at an offset of input 2 × strength (u16 × 16) >> 12 along the angle of input 1: v >> 4 & 0xff
for one channel, v × 255 >> 12 & 0xff for three; through the cos / sin tables, then >> 12); 21
mix (input 2: 4096 → input 0, 0 → input 1, else (4096 − t)·b + t·a >> 12); 22 invert (4096 − v);
30 range (v × (high − low) >> 12 + low, default 1024..3072); 32 emboss (slopes of input 0 across
the neighbouring rows and columns × scale × width × 32 >> 12, a table 255 / √((y² + x² + 65535)
/ 65535) as float, lit by a direction from two angles made length 4096); 34 noise and 15 cells
(below); 36, 39 and 18 read pictures (below). `--render-texture <id>` writes a texture at
128 × 128 and counts the drawable ones at 16 × 16: 2312 of 2315. The other 3 (1446, 1497,
1646) have ellipses wider than a 16-pixel picture with outlines wider than themselves, which
the reference also writes past the edge of at that size; at 128 they draw. 13 (`MaterialProp13`,
one channel, no settings): per pixel `method15428`(x, y) % 4096 with n = 57y + x, n ^= n << 1,
4096 − (((n·n·15731 + 789221)·n + 1376312589) & 0x7fffffff) / 262144 in Java ints (the
remainder keeps the sign). 20 (`MaterialProp20`, settings 0 tiles across and 1 down, u8,
default 4 and 4): tiles of width / across by height / down; row y reads input 0's row
(y % tileH)·height / tileH (0 when tileH is 0), column x reads (x % tileW)·width / tileW (0 when
tileW is 0); a count of 0 divides by zero in the reference and isn't drawn here.

Shapes (operation 29, `MaterialProp29`; `material.rs` `Shape`, `client/shapes.rs`). Setting 0:
a count, then per shape a type byte and its numbers (signed shorts in 0..4096 of the texture,
colours u24): 0 a line (x1, y1, x2, y2, colour, a byte), 1 a cubic curve (four points, colour,
a byte), 2 a rectangle (x1, y1, x2, y2, fill, outline, outline width), 3 an ellipse (centre x,
y, radius x, y, fill, outline, width); another type reads nothing and leaves a null the
reference can't draw. Setting 1: one channel (byte == 1). The whole picture is drawn at once,
black to start, into the op's row cache, which must keep every row (`anInt7668` 255 or the
height; the reference throws otherwise); one channel holds the RGB colours as they are, three
split them (red (c & 0xff0000) >> 12 ...). Each number is scaled × size >> 12. The clip is
0..=width − 1, 0..=height − 1 (`WallDecoration.method16086`). Rectangles and ellipses always
have both colours (u24 is never negative), so they take the fill-and-outline routines; lines
and curves take the outline ones. Every routine fills rows with `method3922` (x1..=x2
inclusive), so the outline spans overlap the fill by a pixel: the right edge comes out one
column wider than the left.
- Line (`QuickChatMessage.method6159`): straight ones as a column (`method13411`: the lower y up
  to, not including, the higher) or a row; others clipped along the 20.12 slope
  (dy << 12) / dx, then Bresenham (`AnimationIndexLoader.method11220`; steep lines walked along
  y, error starting at −(run >> 1)).
- Curve (`Node_Sub24.method12399`): 32 straight pieces, t every 128 of 4096 with t² and t³ >> 12
  and the cubic's coefficients (x3 − 3x2 + 3x1 − x0, 3x0 − 6x1 + 3x2, 3x1 − 3x0); unclipped
  Bresenham pieces when all four points are inside, clipped lines otherwise; a curve whose ends
  are its control points is one line.
- Rectangle (`Ground.method6731`): `width` rows of outline at the top and bottom, then each row
  between: outline x1..x1 + width, fill to x2 − width, outline to x2; clamped edges when it
  crosses the picture (`PlayerModel.method4034`).
- Ellipse (`Class299.method5316`): equal radii are a circle (`Class155.method2637` inside,
  `Class60.method1174` clipped: midpoint circles for r and the inner r − width, the inner x per
  row kept in `Class5.anIntArray36`, which the reference shares between all circles and is
  fresh per circle here); others a midpoint ellipse of the outer and inner radii
  (`MaterialProp30.method15241` inside, `ClanVarDefinitions.method6824` clipped), in Java ints.

26 (`MaterialProp26`, one channel; settings 0 low and 1 high, u16, default 0 and 4096): 4096
where input 0 is within low..=high, else 0. 35 (`MaterialProp35`, one channel; setting 0
strength, u16, default 4096): from input 0's rows above and below (wrapping) and the pixels
either side, dy = (below − above)·strength >> 12, dx = (right − left)·strength >> 12, length =
(int)(√((dy² >> 12) + (dx² >> 12) + 4096) / 4096f) × 4096), out 4096 − 16777216 / length (0 for
length 0). 38 (`MaterialProp38`, one channel; settings 0 seed u8, 1 count u16 (2000), 2 length
u8 (16), 3 angle u16 (0), 4 spread u16 (4096)): the whole picture at once, black to start, like
29 (the row cache must keep every row). With `Random(seed)`, per scratch: angle + (method7931
(spread) − spread / 2) when spread > 0, >> 4 & 0xff into the cos / sin tables; a start point
method7931(width), method7931(height); the end start + cos·length >> 12, sin·length >> 12;
nothing when they're equal. Then a Bresenham line along the longer axis from the lower end,
leaving out the far end, error starting at −run / 2; value 1024 − (method7931(4096) >> 2) +
2048 / run per pixel + 1024; written at [major & (width − 1)][minor & (height − 1)] for a
shallow line and [minor & (height − 1)][major & (width − 1)] for a steep one (the reference's
own indexing: the row comes from x either way).

27 (`MaterialProp27`, one channel; settings 0 count u8 (10), 1 width u16 (2048), 2 direction
u8 (0)): `method14842` splits 0..4096 into `count` stripes of step = 4096 / count (a count of 0
divides by zero), stripe i starting at i·step and on until i·step + (step·width >> 12); one more
start at 4096 and end at the first end + 4096. A position's value is 4096 when it falls in a
stripe's [start, next start) before that stripe's end, else 0. Direction 0: the row's y, the
whole row the same; 1: x; 2: ((x − (4096 − y)) >> 1) + 2048; 3: ((x − y) >> 1) + 2048; any other
direction tests 0.

4 (`MaterialProp4`, bricks, one channel; settings 0 columns u8 (4), 1 rows u8 (8), 2 column
jitter u16 (409), 3 row jitter u16 (204), 4 row offset u16 (1024), 5 y shift u16 (0), 6 mortar
u16 (81), 7 shade range u16 (1024)). Made ready (`method15422`) with `Random(rows)` (the row
count is the seed): brick width 4096 / columns and height 4096 / rows (0 divides by zero);
row r > 0 starts at the last start + height + ((((method7931(4096) − 2048)·row jitter >> 12)·
(height / 2)) >> 12); in each row, brick c > 0 starts at the last + width + the same with the
column jitter and width / 2, then its shade 4096 − method7931(shade range) (4096 without a
range), drawn in that order (row jitter, then each brick's jitter and shade); 4096 after the
last row and the last brick of each row. A row: y + y shift wrapped into 0..=4096 (add 4096
while below 0, take 4096 while above 4096); its row is the last start ≤ y; inside the row's
mortar margins (start + mortar / 2 < y < next start − mortar / 2) each x is shifted by
±offset·width >> 12 (+ for an even row count i, − for odd), wrapped the same way, and inside
its brick's margins takes the brick's shade; mortar is 0.

11 (`MaterialProp11`, three channels; settings 0..=2 the tint red, green, blue, u16, default
4096): a grey input pixel (red = blue and green = blue) becomes its level × tint >> 12 per
channel, any other pixel the tint itself. 12 (`MaterialProp12`, waves, one channel; settings
0 round (u8, 0 straight), 1 wave (u8: 0 sine, 2 triangle, else saw), 3 frequency (u8, 1);
setting 2 reads nothing): straight v = (x − y)·frequency; round v = (int)((int)(√(((y − 2048)
>> 1)² + ((x − 2048) >> 1)² >> 12) / 4096f) × 4096)·frequency × π); then v −= v & −4096 (its
place in a 4096 wave); sine (sin[v >> 4 & 0xff] + 4096) >> 1, triangle (2048 − |v − 2048|) << 1,
saw v. 23 (`MaterialProp23`, kaleidoscope; setting 0 one channel (byte == 1)): each pixel reads
input 0 at a point picked by the angle of its offset from the centre, atan2(x − 2048, y − 2048)
as a float, compared with the eighths of a turn as doubles in order: [−π, −3π/4] (x, y);
[−3π/4, −π/2] (y, x); [−π/2, −π/4] (width − y, x); [−π/4, 0] (x, height − y); [0, π/4] (width −
x, height − y); [π/4, π/2] (width − y, height − x); [π/2, 3π/4] (y, height − x); [3π/4, π]
(width − x, y); then masked to the picture. Straight above the centre the float π is past the
double π, so no eighth matches and the pixel reuses the point before it (static fields in the
reference).

28 (`MaterialProp28`, blocks, one channel; settings 0 seed u8, 1..=2 width range u16 (1024,
2048), 3..=4 height range u16 (409, 819), 5 bevel u16 (1024), 6 bevel mode u8 (0), 7 bevel
randomness u16 (1024), 8 shade range u16 (1024)): the whole picture at once, black to start,
like 29 (the row cache must keep every row). Sizes × width or height >> 12; a top height ≤ 1
draws nothing. Bevel = (width / 8)·bevel >> 12. With `Random(seed)`, blocks are laid row by
row: each block's width is min + method7931(max − min), its height the same (an empty range
throws), cut at the right edge; the first row sits on 0; a later row starts at a random x
(method7931(width)), and each block rests on the highest top among the blocks of the row
below under it (found from the one under its left end, both ends shifted by the row's offset
and wrapped), the lower ones there each getting a filler block up to that top first (spanning
where they overlap). A block that would pass the top is cut to it; when every block of a row
was cut the picture is done. A block (`method15253`): shade 4096 − method7931(shade range)
(4096 without), edge = bevel − method7931(bevel randomness·bevel >> 12) (when that's > 0); x
wraps; with an edge, ex = min(width / 2, edge) and ey = min(height / 2, edge): rows within ey
of the top or bottom have level shade·distance / ey, the others the shade; the first and last
ex columns ramp shade·i / ex (mode 0 × level >> 12, else the lower of the two; full rows just the
ramp); the middle takes the level (wrapping round the right edge). Without an edge, the block
is filled with its shade, wrapped. The reference loops for ever on some broken settings; this
stops and refuses.

The last ones. 14 (`MaterialProp14`, frame, one channel; setting 0 the inner edge e, u16,
585), with d = 2048 − e (0 divides by zero): inside the horizontal bar (e < x < 4096 − e and
2048 − e < y < 2048 + e) 4096 − (|2048 − x| << 12) / d; else inside the vertical bar's x range
(|y − 2048| − e) << 12 / d; else for e ≤ y ≤ 4096 − e, 0 inside e..=4096 − e in x and
4096 − (|2048 − y| << 12) / d outside it; else (|x − 2048| − e) << 12 / d. 16 (`MaterialProp16`,
weave, one channel; settings 0 threads across u8 (1), 1 down u8 (1), 2 thickness u16 (204)):
tx = x·across >> 12, ty = y·down >> 12, ix = (x % (4096 / across))·across, iy the same down;
where iy < thickness the thread is on top only when (tx − ty) wrapped into 0..=3 is 1 and
ix ≥ thickness; then where ix < thickness only when (tx − ty) wrapped is 0 (the first wrap
carries over); 4096 on top, else 0. 24 (`MaterialProp24`, one channel): (blue + green + red) /
3 of input 0. 25 (`MaterialProp25`, colour key, three channels; settings 0 closeness u16 (409),
1..=3 blue, green and red scale u16 (4096), 4 the key u24, kept as red (c & 0xff0000) << 4,
green (c & 0xff00) >> 4, blue (c & 0xff) >> 12 — the reference's shifts, so red is far out of
range and blue always 0): a pixel within the closeness of the key on red, then green, then blue
is scaled channel by channel (× scale >> 12), any other kept. 37 (`MaterialProp37`, lattice,
one channel; settings 0..=6 u16: x and y offsets of the first set (2048, 0), of the second (0,
2048), frequency (12288), width (4096), divisor (8192)): x − 2048 and y − 2048 plus each set's
offsets, wrapped into −2048..=2048 (once each way); a set's half-width at a point is
((cos[(along·frequency >> 12)·255 >> 12 & 0xff] << 12) / frequency << 12) / divisor·width >> 12,
`along` b − a for the first set (inside when |b + a| is within it) and d + c for the second
(inside when |d − c| is); 4096 inside either, else 0.

Pictures (`texture.rs` `pictures`, `pixels`, `Render::rgb`). Before the rows
(`MaterialProperty.method12315`), each 36 (`MaterialPropTexture`, setting 0 the texture id, u16)
renders its texture at 64 × 64 when the table marks it half size, else 128 × 128, through
`MaterialDefinitions.method14718` at brightness 1.0: the opaque output only, rows written left
to right (a brick tile column by column), each channel >> 4 clamped through the table, alpha 255
where the colour isn't 0. Each 39 (`MaterialPropSprite`) and 18 (`MaterialPropSpriteSub18`),
setting 0 the sprite id (u16), loads index 8's file id (`SpriteDefinitions.method1515`, the
first sprite; `SPRITE_ARCHIVE_ID` is −1) and places it in its full frame (`load`), palette colour
per pixel (`getPixels`); without a sprite id its rows stay 0. A row of 36 or 39: source row
h·y / height (y when the heights match), column x·w / width (x when the widths match); channels
red (c & 0xff0000) >> 12, green (c & 0xff00) >> 4, blue (c & 0xff) << 4. A row of 18 starts at
(y % h) × h (h, not w, as the reference has it) and reads x % w. A texture that reads itself
has no end in the reference; reads deeper than 8 are refused here.

Noise (`material.rs` `noise`, `JavaRandom`, `permutation`; `texture.rs` `Render::noise`,
`Render::cells`). Both seed a `java.util.Random` with their seed setting. The shuffled table
(`Class316.method5588`): 512 bytes, entries 0..255 set to 0..254 (entry 255 stays 0), then for i
in 0..255 with j = 255 − i: k = `Class476.method7931`(random, j); the bytes at k and j swap and
the one put at j is copied to 511 − i (entry 256 stays 0). `method7931(n)`: a power of two →
(nextInt as unsigned × n) >> 32; else draw until nextInt < MIN_VALUE − (2³² mod n), then
`Class302.method5360` ((v + (v >>> 31)) % n + (v >> 31 & n − 1)). The fade curve
(`Class370.method6326`) for t in 0..4096: c = t·(t·t >> 12) >> 12, a = 6t − 61440,
b = (t·a >> 12) + 40960, fade = b·c >> 12.
- 34 (`MaterialProp34`): settings 0 centred (byte == 1, default on), 1 octaves (default 4), 2
  persistence (short, default 1638; negative: the current octave count's strengths follow as
  shorts), 3 both cell counts (default 4), 4 seed, 5 cells across, 6 cells down. Made ready
  (`method15199`): persistence > 0 → strength i = (short)(int)(pow(p / 4096f, i) × 4096); the
  frequencies are (short)(int)2^i; the last octaves of strength −8..=8 are dropped (never the
  first). A row: for each octave of strength s (skipped when −8..=8 and there are several; the
  reference then leaves the row cache's old values, 0 here), f = freq << 12, the row position
  y·cellsDown·f >> 12 gives a cell (next wraps at f·cellsDown >> 12) and a fraction; each pixel's
  x·cellsAcross·f >> 12 goes through `method15198` (gradient noise over the four corners, each
  corner's gradient from table & 3: (1,1), (−1,1), (1,−1), (−1,−1), blended by the fade curve)
  and adds v·s >> 12; after the last octave a centred row is (v >> 1) + 2048.
- 15 (`MaterialProp15`): settings 0 both cell counts (default 5), 1 seed, 2 jitter (u16, default
  2048), 3 output (default 2), 4 distance (default 1), 5 across, 6 down. The 512 point offsets come
  from another `Random(seed)`, each `method7931(jitter)` as a short (all 0 without jitter). Each
  pixel (x·across + 2048, y·down + 2048) looks at the 3 × 3 cells around it (cell indexes past the
  count wrap once), the point of cell (i, j) at offsets[2·table[(table[j] + i) & 0xff]] plus
  (i << 12, j << 12), and keeps the four smallest distances: 1 dy² + dx² >> 12, 2 |dx| + |dy|, 3
  max, 4 (√(|dx| / 4096f) + √(|dy| / 4096f))·4096 squared >> 12, 5 ∜((dx² + dy²) / 16777216f)
  × 4096, else √((dx² + dy²) / 16777216f) × 4096. Output 0 nearest, 1 second, 2 second − nearest,
  3 third, 4 fourth (another value leaves the row as it was, 0 here).

### Roofs hidden over the player (`client/scene.rs`, `client/world.rs`)
Every ground tile and object is a piece with its tile slot: its plane, one lower on a bridge tile
(`tileMasks[1] & 2`; `SceneObjectManager.method3382` moves each slot down and drops the plane of
what is anchored there). The roof mode is `removeRoofsOptionOverride`, set from the remove-roofs
preference when the game starts (`Class190`): 0 nothing is hidden; 1 every roof tile is flooded
once, at most 512 floods (`Class17.method570`); 2 each frame (`QuickchatMessageLoader.method7215`,
the game camera's mode 2), not on plane 3: from the player's tile if it is a roof tile
(`tileMasks & 4`), else, with the drawn pitch below 2560, from the first roof tile on the walk
from the camera's tile (kept inside the scene) to the player's (one axis a tile a step, the other
by a 65536-scaled slope starting at 32768). The flood (`JS5Manager.method5492`) marks the start
and spreads, from roof tiles only, to the 4 sides and to the diagonals whose two shared sides
aren't roof tiles, carrying edge codes (type | rotation << 6: west 0x12/0xd3/0x13, north-west
0x12/0x52/0x13, north 0x52/0x13/0x53, north-east 0x52/0x92/0x53, east 0x92/0x53/0x93, south-west
0xd2/0x12/0xd3, south 0xd2/0x93/0xd3, south-east 0x92/0xd2/0x93). On each popped tile, every plane
above the player's without `tileMasks & 8` is marked (with the whole span of a bigger scenery
object there), except on a border tile (not a roof tile) where a wall faces one of the codes
(`Class125.method2172`: 18 → 1, 2, 4, 8 by rotation; 19 and 21 → 16, 32, 64, 128; walls'
orientation `Class235.method3967`: corners 1 and 3 → 16..128, others 1..8) or a scenery object's
code equals one (21 counted as 19). Drawing (`method3424`, `method3441`): a piece on slot p ≥
player plane + 1 is hidden where its tile is marked (scenery: every tile of its span). The
self-test stands the player on the nearest roof tile: 249 pieces above hide. Not yet: the hidden
roofs' bounds (`anIntArray7198`...), occlusion culling (`Class201`), the tile under a bridge
(`aClass293_3509`), and the camera's bridge lift (`getTerrainHeightAtPos` +1 plane).

Walls (`method12464`): types 0, 1, 3 and 9 are one piece at the location's rotation; a whole
corner (2) is two, rotation + 4 (so mirrored) and (rotation + 1) & 3. Walls 0..=3 become the
tile's wall (`SceneObjectManager.getWall`, the last one placed so far, in file order). Wall
decorations (`method12465`) all use the type-4 models (`Class485.method8201`): 4 at the rotation;
5 stood off the wall by d = its `decorDisplacement` (opcode 28 × 4, default 64) + 1, or 65 with no
wall, × (1, 0), (0, −1), (−1, 0), (0, 1) by rotation; 6 at rotation + 4, d = displacement / 2 + 1
(33 with no wall) × the same table when still, × (1, −1), (−1, −1), (−1, 1), (1, 1) when animated;
7 at ((rotation + 2) & 3) + 4; 8 two pieces, rotation + 4 off by d × the diagonal table, and
((rotation + 2) & 3) + 4. The offset moves the model when it is drawn, after ground fitting
(`GraphNode_Sub1_Sub4_Sub1`).

### The game camera (`client/camera.rs`)
Angles are 16384 to a turn (`Trig.SINE`/`COSINE`, ×16384). Starts at pitch `camAngleX` 1052, yaw 0,
`CAMERA_ZOOM` 600 (`client.java`, `HitbarDefinitions`). Each frame (`Isaac.processCamera`): pitch
clamped to 1081..=2980, yaw wrapped; the lift `anInt7273` moves towards ((highest rise of the
ground 4 tiles around the focus above the focus) >> 2) × 1536, clamped 262144..=786432, by 1/24
going up and 1/80 coming down. Placement (`LoadingStage.method6683`): pitch = max(camAngleX,
lift >> 8); distance ((pitch >> 3) × 3 + zoom) << 2, × ((320 − 256) × s / 100 + 256) >> 8 with
s = clamp(viewport height − 334, 0, 100); with a = (16384 − pitch) & 0x3fff and b = (16384 − yaw)
& 0x3fff, the eye is the focus (200 above the ground there, `Ground.averageHeight`) minus
(sin(b)·cos(a)·d, −sin(a)·d, cos(b)·cos(a)·d) in fixed point. Focal length (`Class492.method8265`,
`anInt7451`): viewport height × ((205 − 256) × s / 100 + 256) / 334. Ctrl + mouse wheel zooms by
30 within −50..1700 (`Class209_Sub1.mouseWheelMoved`). Not yet: turning (it comes from the game
interface's scripts), the focus easing after a moving player, shake, the bridge plane in the lift.
