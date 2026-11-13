//! Developer tools that read the local cache directly (no server or window).
//!
//! - `--dump-interface <id>`: every component of an interface with its kind, parent, box,
//!   content and script hooks — the starting point for porting a screen.
//! - `--dump-script <id>`: a CS2 script's instructions.
//! - `--render-interface <id>`: opens an interface as the root window from the local cache,
//!   runs its on-load scripts and saves the drawing to `shots/interface-<id>.png`. With
//!   `RENDER_INPUT="click 382 298; type name; key 84"` it first plays those clicks, typed text and
//!   client key codes through the interface's input hooks, one tick each.
//! - `--scripts-using <n>`: every CS2 script with `n` as an int operand (`PUSH_INT n`,
//!   `CALL_CS2 n`, …) — how to find who calls a script or touches a component (`n` = full id).
//! - `--export-music <id>`: rebuilds music track `id` (index 6) as `shots/music-<id>.mid` and
//!   names the login screen's track (defaults file 4).
//! - `--render-music <id>`: plays music track `id` through the MIDI player (only the samples it
//!   uses are loaded) and saves the first 60 seconds as `shots/music-<id>.wav`.
//! - `--script-scope <id>`: every script reachable from one through `CALL_CS2`, and the opcodes
//!   they use — what the interpreter must support to run it.

use crate::cache::cs2::{Operand, Script, CALL_CS2};
use crate::cache::interface::{decode_interface, Component, Content};
use crate::cache::Cache;
use crate::config::Settings;
use std::collections::{BTreeMap, BTreeSet};

const INTERFACE_INDEX: u8 = 3;
const SCRIPT_INDEX: u8 = 12;

fn load_script(cache: &mut Cache, id: u32) -> Result<Script, String> {
    let file = cache.read_files(SCRIPT_INDEX, id).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    Script::decode(&file).map_err(|e| format!("script {id}: {e}"))
}

pub fn dump_script(settings: &Settings, id: u32) -> Result<(), String> {
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    cache.load_table(SCRIPT_INDEX).map_err(|e| e.to_string())?;
    let s = load_script(&mut cache, id)?;
    println!(
        "script {id} {:?}: {} instructions, args int {} string {} long {}, locals int {} string {} long {}",
        s.name,
        s.instructions.len(),
        s.int_args,
        s.string_args,
        s.long_args,
        s.int_locals,
        s.string_locals,
        s.long_locals
    );
    for (n, table) in s.switches.iter().enumerate() {
        let mut cases: Vec<_> = table.iter().collect();
        cases.sort();
        println!("switch {n}: {cases:?} (value, jump)");
    }
    for (n, i) in s.instructions.iter().enumerate() {
        let operand = match &i.operand {
            Operand::Int(v) => v.to_string(),
            Operand::Text(t) => format!("{t:?}"),
            Operand::Long(v) => format!("{v}L"),
        };
        println!("{n:>4} {:<28} {operand}", i.name());
    }
    Ok(())
}

pub fn script_scope(settings: &Settings, id: u32) -> Result<(), String> {
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    cache.load_table(SCRIPT_INDEX).map_err(|e| e.to_string())?;
    let mut seen = BTreeSet::new();
    let mut todo = vec![id];
    let mut opcodes: BTreeMap<&'static str, usize> = BTreeMap::new();
    while let Some(next) = todo.pop() {
        if !seen.insert(next) {
            continue;
        }
        let script = load_script(&mut cache, next)?;
        for i in &script.instructions {
            *opcodes.entry(i.name()).or_default() += 1;
            if i.opcode == CALL_CS2 {
                todo.push(i.int() as u32);
            }
        }
    }
    println!("{} scripts reachable from {id}: {seen:?}", seen.len());
    println!("{} different opcodes:", opcodes.len());
    for (name, count) in opcodes {
        println!("  {name:<32} {count}");
    }
    Ok(())
}

pub fn dump_interface(settings: &Settings, id: u32) -> Result<(), String> {
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    cache.load_table(INTERFACE_INDEX).map_err(|e| e.to_string())?;
    let files = cache.read_files(INTERFACE_INDEX, id).map_err(|e| e.to_string())?;
    let components = decode_interface(id, &files).map_err(|e| e.to_string())?;
    println!("interface {id}: {} components", components.len());
    for c in &components {
        println!("{}", describe(c));
    }
    Ok(())
}

fn describe(c: &Component) -> String {
    let parent = c.parent.map_or("root".to_string(), |p| format!("{}", p & 0xffff));
    let content = match &c.content {
        Content::Text { font, text, colour, h_align, v_align, shadow, .. } => {
            format!("text font={font} colour={colour:#x} align={h_align}/{v_align} shadow={shadow} {text:?}")
        }
        Content::Sprite { sprite, tiling, transparency, .. } => format!("sprite {sprite} tiling={tiling} alpha={transparency}"),
        Content::Figure { colour, filled, transparency } => format!("figure colour={colour:#x} filled={filled} alpha={transparency}"),
        Content::Model { model, animation } => format!("model {model} anim {animation}"),
        Content::Line { width, colour, .. } => format!("line w={width} colour={colour:#x}"),
        Content::Container { scroll_width, scroll_height } => format!("container scroll {scroll_width}x{scroll_height}"),
        Content::None => format!("{:?}", c.kind),
    };
    let hooks: Vec<i32> = c.other_hooks.iter().chain([&c.on_mouse_over, &c.on_mouse_leave]).filter_map(Component::script_of).collect();
    format!(
        "#{:<3} parent {:<4} {}{} at ({},{}) size {}x{} modes w{} h{} x{} y{} | {content} | onload {:?} | hooks {:?} | settings {:#x} options {:?} varps {:?}",
        c.id & 0xffff,
        parent,
        if c.hidden { "HIDDEN " } else { "" },
        c.name.clone().unwrap_or_default(),
        c.x,
        c.y,
        c.width,
        c.height,
        c.width_mode,
        c.height_mode,
        c.x_mode,
        c.y_mode,
        c.on_load,
        hooks,
        c.settings,
        c.options,
        c.varps
    )
}

pub fn render_interface(settings: &Settings, id: u32) -> Result<(), String> {
    use crate::client::interface::assets::Assets;
    use crate::client::source::Archives;
    use crate::client::window::Window;
    let cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    let mut window = Window::open(Assets::new(Archives::new(Box::new(cache))), id, None)?;
    for step in std::env::var("RENDER_INPUT").unwrap_or_default().split(';').map(str::trim).filter(|s| !s.is_empty()) {
        window.ui.play(step)?;
    }
    if let Some((name, password)) = &window.ui.login_request {
        println!("the login window asked to log in as {name:?} with a {}-character password", password.chars().count());
    }
    let p = &window.ui.prefs;
    println!("volumes: music {}, sound {}, area {}, voice {}", p.music_volume, p.sound_volume, p.ambient_volume, p.voice_volume);
    window.render();
    let path = std::path::PathBuf::from(format!("shots/interface-{id}.png"));
    std::fs::create_dir_all("shots").map_err(|e| e.to_string())?;
    window.raster.save_png(&path)?;
    let lit = window.raster.pixels.iter().filter(|&&p| p != 0).count();
    println!("saved {} ({lit} lit pixels)", path.display());
    Ok(())
}

pub fn export_music(settings: &Settings, id: u32) -> Result<(), String> {
    use crate::cache::music;
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    for index in [6, 28] {
        cache.load_table(index).map_err(|e| e.to_string())?;
    }
    let defaults = cache.read_files(28, 4).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    println!("login music track: {:?}", music::login_track(&defaults).map_err(|e| e.to_string())?);
    let packed = cache.read_files(6, id).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    let midi = music::to_midi(&packed).map_err(|e| format!("track {id}: {e}"))?;
    // `Node_Sub7.method12165`: the notes each program plays — only those samples get loaded.
    let used = crate::cache::midi::used_notes(&midi);
    let notes: usize = used.values().map(|n| n.iter().filter(|&&b| b).count()).sum();
    println!("track {id} plays {} programs, {notes} distinct program notes: {:?}", used.len(), used.keys().collect::<Vec<_>>());
    // Every patch (MIDI program) must decode; report what the samples need.
    cache.load_table(15).map_err(|e| e.to_string())?;
    let ids: Vec<u32> = cache.table(15).map(|t| t.archives.iter().map(|a| a.id).collect()).unwrap_or_default();
    let (mut vorbis, mut synth) = (BTreeSet::new(), BTreeSet::new());
    for patch_id in &ids {
        let data = cache.read_files(15, *patch_id).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
        let p = crate::cache::music_patch::Patch::decode(&data).map_err(|e| format!("patch {patch_id}: {e}"))?;
        for s in p.samples.iter().filter(|&&s| s != 0) {
            let s = s - 1;
            if s & 1 == 1 {
                vorbis.insert(s >> 2);
            } else {
                synth.insert(s >> 2);
            }
        }
    }
    println!(
        "{} patches decoded; they use {} Vorbis samples (index 14) and {} synthesized sounds (index 4)",
        ids.len(),
        vorbis.len(),
        synth.len()
    );
    // Decode every Vorbis sample with the shared setup (index 14, file 0/0).
    cache.load_table(14).map_err(|e| e.to_string())?;
    let setup_data = cache.read_files(14, 0).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    let setup = crate::cache::vorbis::Setup::decode(&setup_data).map_err(|e| format!("vorbis setup: {e}"))?;
    let (mut ok, mut frames, mut first) = (0, 0usize, None);
    for &sample_id in &vorbis {
        // `Class250.method4287`: one file per archive here, so the sample is archive `id`, file 0.
        let data = cache.read_files(14, sample_id as u32).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
        match crate::cache::vorbis::decode_sample(&setup, &data) {
            Ok(s) => {
                ok += 1;
                frames += s.pcm.len();
                first.get_or_insert((sample_id, s));
            }
            Err(e) => eprintln!("vorbis sample {sample_id}: {e}"),
        }
    }
    println!("{ok} of {} Vorbis samples decoded ({frames} sample frames)", vorbis.len());
    // The synthesized sounds (index 4), as `Class250.method4290` addresses them.
    cache.load_table(4).map_err(|e| e.to_string())?;
    let single = cache.table(4).is_some_and(|t| t.archives.len() == 1);
    for &sound in &synth {
        let (archive, file) = if single { (0, sound as usize) } else { (sound as u32, 0) };
        let data = cache.read_files(4, archive).map_err(|e| e.to_string())?.into_iter().nth(file).unwrap_or_default();
        match crate::cache::sound_effect::synthesize(&data) {
            Ok(s) => {
                let peak = s.pcm.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
                println!("synth sound {sound}: {} samples, loop {}..{}, peak {peak}", s.pcm.len(), s.loop_start, s.loop_end);
            }
            Err(e) => eprintln!("synth sound {sound}: {e}"),
        }
    }
    std::fs::create_dir_all("shots").map_err(|e| e.to_string())?;
    if let Some((sid, s)) = first {
        let path = format!("shots/sample-{sid}.wav");
        std::fs::write(&path, wav(&s.pcm, s.rate as u32)).map_err(|e| e.to_string())?;
        println!("saved {path} ({} Hz, {} samples, loop {}..{}, flag {})", s.rate, s.pcm.len(), s.loop_start, s.loop_end, s.loop_flag);
    }
    let path = format!("shots/music-{id}.mid");
    std::fs::write(&path, &midi).map_err(|e| e.to_string())?;
    println!("saved {path} ({} bytes)", midi.len());
    Ok(())
}

/// 16-bit mono WAV of float samples (for listening to a decoded sample).
fn wav(pcm: &[f32], rate: u32) -> Vec<u8> {
    let data: Vec<u8> = pcm.iter().flat_map(|&v| (((v * 32767.0).clamp(-32768.0, 32767.0)) as i16).to_le_bytes()).collect();
    let mut out = Vec::with_capacity(44 + data.len());
    out.extend(b"RIFF");
    out.extend((36 + data.len() as u32).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * 2).to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(data);
    out
}

pub fn scripts_using(settings: &Settings, value: u32) -> Result<(), String> {
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    cache.load_table(SCRIPT_INDEX).map_err(|e| e.to_string())?;
    let ids: Vec<u32> = cache.table(SCRIPT_INDEX).map(|t| t.archives.iter().map(|a| a.id).collect()).unwrap_or_default();
    let mut found = Vec::new();
    for id in ids {
        let Ok(script) = load_script(&mut cache, id) else { continue };
        let hits: Vec<String> = script
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, i)| matches!(i.operand, Operand::Int(v) if v as u32 == value))
            .map(|(pc, i)| format!("{pc}:{}", i.name()))
            .collect();
        if !hits.is_empty() {
            found.push(format!("script {id}: {}", hits.join(", ")));
        }
    }
    println!("{} scripts use {value}:", found.len());
    for line in found {
        println!("  {line}");
    }
    Ok(())
}

/// `--render-music <id>`: song `id` played once, as the client plays it, into
/// `shots/music-<id>.wav`.
pub fn render_music(settings: &Settings, id: u32) -> Result<(), String> {
    use crate::client::music::{load, RATE};
    use crate::client::source::Archives;
    let cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    let mut archives = Archives::new(Box::new(cache));
    let single_synth = archives.archive_count(4) == Some(1);
    let packed = archives.archive(6, id).and_then(|f| f.first().cloned().flatten()).ok_or(format!("track {id} isn't in the cache"))?;
    let midi = crate::cache::music::to_midi(&packed).map_err(|e| format!("track {id}: {e}"))?;
    let song = load::gather(&mut |index, archive| archives.archive(index, archive), single_synth, midi)?;
    let started = std::time::Instant::now();
    // The login music volume the client starts with (`aPreference_Sub13_8229`, 127).
    let pcm = song.render(true, crate::client::preferences::Preferences::default().sub13_volume, 600)?;
    let peak = pcm.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    std::fs::create_dir_all("shots").map_err(|e| e.to_string())?;
    let path = format!("shots/music-{id}.wav");
    std::fs::write(&path, load::wav(&pcm, true)).map_err(|e| e.to_string())?;
    println!(
        "saved {path}: {:.1} s stereo at {RATE} Hz, peak {peak}, rendered in {:.1} s",
        pcm.len() as f64 / 2.0 / RATE as f64,
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// `--texture-stats <any>`: the texture table (index 26) and every texture's material (index 9):
/// how many decode, and how many textures use each operation. A check of the data, to see which
/// operations drawing needs first.
pub fn texture_stats(settings: &Settings, _: u32) -> Result<(), String> {
    use crate::cache::material::{texture_table, Material};
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    cache.load_table(26).map_err(|e| e.to_string())?;
    cache.load_table(9).map_err(|e| e.to_string())?;
    let table = cache.read_files(26, 0).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    let details = texture_table(&table).map_err(|e| e.to_string())?;
    let single = cache.table(9).map_or(0, |t| t.archives.len()) == 1;
    let all = if single { cache.read_files(9, 0).map_err(|e| e.to_string())? } else { Vec::new() };
    let (mut decoded, mut failed) = (0, Vec::new());
    let mut kinds: BTreeMap<u8, usize> = BTreeMap::new();
    let mut ground = 0;
    for (id, d) in details.iter().enumerate() {
        let Some(d) = d else { continue };
        ground += usize::from(d.ground_mesh);
        let data = if single { all.get(id).cloned() } else { cache.read_files(9, id as u32).ok().and_then(|f| f.into_iter().next()) };
        let Some(data) = data else { continue };
        match Material::decode(&data) {
            Ok(m) => {
                decoded += 1;
                let used: BTreeSet<u8> = m.ops.iter().map(|o| o.kind).collect();
                for k in used {
                    *kinds.entry(k).or_default() += 1;
                }
                // Every output names an operation; images name their sprite or texture.
                let images = m.ops.iter().filter(|o| o.image >= 0).count();
                if id < 3 {
                    println!(
                        "texture {id}: {} operations, outputs {:?}, {images} images, inputs {:?}",
                        m.ops.len(),
                        m.outputs,
                        m.ops.iter().map(|o| &o.inputs).collect::<Vec<_>>()
                    );
                }
            }
            Err(e) => failed.push(format!("{id}: {e}")),
        }
    }
    println!(
        "{} textures in the table ({} defined, {ground} ground mesh); {decoded} materials decode, {} fail",
        details.len(),
        details.iter().flatten().count(),
        failed.len()
    );
    for f in failed.iter().take(10) {
        println!("  failed {f}");
    }
    println!("operations by how many textures use them: {kinds:?}");
    let first = details.iter().flatten().next();
    if let Some(d) = first {
        println!(
            "first texture: colour {}, brightness {}, shadow {}, brick {}, blend {}, skip {}",
            d.colour, d.brightness, d.shadow, d.brick, d.blend, d.skip_triangles
        );
    }
    Ok(())
}

/// `--render-texture <id>`: texture `id`'s pixels at 128 × 128 (`renderIntPixels`) in
/// `shots/texture-<id>.png`, and how many of all the textures can be drawn so far.
/// Textures and sprites read from the cache, for `--render-texture`.
struct CacheImages {
    cache: Cache,
    details: Vec<Option<crate::cache::material::TextureDetails>>,
    files: Vec<Option<Vec<u8>>>,
}

impl CacheImages {
    fn material(&self, id: usize) -> Option<crate::cache::material::Material> {
        self.files.get(id)?.as_ref().and_then(|d| crate::cache::material::Material::decode(d).ok())
    }
}

impl crate::client::texture::Images for CacheImages {
    fn texture(&mut self, id: i32) -> Option<(crate::cache::material::Material, bool, bool)> {
        let d = self.details.get(id as usize)?.clone()?;
        Some((self.material(id as usize)?, d.half_size, d.brick))
    }

    /// `Index.getFile(id)` of index 8, then its first sprite.
    fn sprite(&mut self, id: i32) -> Option<crate::cache::sprite::Sprite> {
        let data = self.cache.read_files(8, id as u32).ok()?.into_iter().next()?;
        crate::cache::sprite::decode(&data).ok()?.into_iter().next()
    }
}

pub fn render_texture(settings: &Settings, id: u32) -> Result<(), String> {
    use crate::cache::material::texture_table;
    use crate::client::texture::render;
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    for index in [8, 9, 26] {
        cache.load_table(index).map_err(|e| e.to_string())?;
    }
    let table = cache.read_files(26, 0).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    let details = texture_table(&table).map_err(|e| e.to_string())?;
    // `Index.getFile(id)`: archive 0's file id when there is one archive, else archive id.
    let single = cache.table(9).map_or(0, |t| t.archives.len()) == 1;
    let files: Vec<Option<Vec<u8>>> = if single {
        cache.read_files(9, 0).map_err(|e| e.to_string())?.into_iter().map(Some).collect()
    } else {
        (0..details.len()).map(|i| cache.read_files(9, i as u32).ok().and_then(|f| f.into_iter().next())).collect()
    };
    let mut images = CacheImages { cache, details, files };
    let mut missing: BTreeMap<u8, usize> = BTreeMap::new();
    let (mut drawn, mut failing) = (0, Vec::new());
    for i in 0..images.details.len() {
        match images.material(i).map(|m| render(&m, 16, false, &mut images)) {
            Some(Ok(_)) => {
                if drawn < 10 {
                    print!("{i} ");
                }
                drawn += 1;
            }
            Some(Err(e)) => {
                *missing.entry(e.0).or_default() += 1;
                failing.push(i);
            }
            None => {}
        }
    }
    println!("{drawn} of {} textures can be drawn; the first operation missing for the rest: {missing:?}", images.details.len());
    println!("first that can't: {:?}", &failing[..failing.len().min(10)]);
    let m = images.material(id as usize).ok_or(format!("no texture {id}"))?;
    for (i, o) in m.ops.iter().enumerate() {
        let params = format!("{:?}", o.params);
        println!(
            "  op {i}: kind {} inputs {:?} {} {}",
            o.kind,
            o.inputs,
            if o.mono { "mono" } else { "rgb" },
            &params[..params.len().min(60)]
        );
    }
    println!("  outputs {:?}", m.outputs);
    let brick = images.details.get(id as usize).cloned().flatten().is_some_and(|d| d.brick);
    let pixels = render(&m, 128, brick, &mut images).map_err(|e| format!("texture {id} needs operation {} (not ported yet)", e.0))?;
    let mut img = image::RgbaImage::new(128, 128);
    for (p, &c) in img.pixels_mut().zip(&pixels) {
        *p = image::Rgba([(c >> 16) as u8, (c >> 8) as u8, c as u8, (c >> 24) as u8]);
    }
    std::fs::create_dir_all("shots").map_err(|e| e.to_string())?;
    let path = format!("shots/texture-{id}.png");
    img.save(&path).map_err(|e| e.to_string())?;
    println!("saved {path}");
    Ok(())
}

/// `--map-locations <region>`: region `(x << 8) | y`'s locations (`l{x}_{y}` in index 5,
/// decrypted with the key from the map keys file), counted by type, then each one with its
/// object's name and how many model sets it has. A check of the data.
pub fn map_locations(settings: &Settings, region: u32) -> Result<(), String> {
    use crate::cache::map_terrain::{archive_name, name_hash};
    use crate::cache::object_def::ObjectDef;
    let (rx, ry) = ((region >> 8) as i32, (region & 0xff) as i32);
    let keys = crate::cache::keys::MapKeys::load(&settings.keys_file).map_err(|e| e.to_string())?;
    let key = keys.by_region().get(&region).copied().unwrap_or_default();
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    cache.load_table(5).map_err(|e| e.to_string())?;
    cache.load_table(16).map_err(|e| e.to_string())?;
    let name = archive_name(false, rx, ry);
    let hash = name_hash(&name);
    let archive = cache
        .table(5)
        .and_then(|t| t.archives.iter().find(|a| a.name_hash == Some(hash)).map(|a| a.id))
        .ok_or(format!("no archive named {name} in index 5"))?;
    let data = cache.read_files_keyed(5, archive, &key).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    let found = crate::cache::map_locations::decode(&data).map_err(|e| e.to_string())?;
    let mut kinds: BTreeMap<u8, usize> = BTreeMap::new();
    for l in &found {
        *kinds.entry(l.kind).or_default() += 1;
    }
    println!("{name} (archive {archive}): {} locations; by type {kinds:?}", found.len());
    for l in &found {
        let file = cache.read_files(16, l.id >> 8).ok().and_then(|f| f.get((l.id & 0xff) as usize).cloned());
        let def = file.and_then(|d| ObjectDef::decode(&d).ok()).unwrap_or_default();
        println!(
            "  {} '{}' at {},{} plane {}: type {} rotation {}, {} model sets",
            l.id,
            def.name,
            l.x,
            l.y,
            l.plane,
            l.kind,
            l.rotation,
            def.models.len()
        );
    }
    Ok(())
}

/// `--map-terrain <region>`: region `(x << 8) | y`'s terrain (`m{x}_{y}`, index 5): what it
/// holds per plane, and a picture seen from above in `shots/map-<region>.png` (one pixel per
/// tile, grey by height, tinted where there is an overlay). A check of the data, not the game's
/// drawing.
pub fn map_terrain(settings: &Settings, region: u32) -> Result<(), String> {
    use crate::cache::map_terrain::{archive_name, name_hash, Terrain, PLANES, SIZE};
    let (rx, ry) = ((region >> 8) as i32, (region & 0xff) as i32);
    let mut cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    cache.load_table(5).map_err(|e| e.to_string())?;
    let name = archive_name(true, rx, ry);
    let hash = name_hash(&name);
    let archive = cache
        .table(5)
        .and_then(|t| t.archives.iter().find(|a| a.name_hash == Some(hash)).map(|a| a.id))
        .ok_or(format!("no archive named {name} in index 5"))?;
    let data = cache.read_files(5, archive).map_err(|e| e.to_string())?.into_iter().next().unwrap_or_default();
    let t = Terrain::decode(&data, rx, ry).map_err(|e| e.to_string())?;
    println!("{name} (archive {archive}, {} bytes)", data.len());
    for plane in 0..PLANES {
        let tiles: Vec<_> = (0..SIZE).flat_map(|x| (0..SIZE).map(move |y| (x, y))).map(|(x, y)| t.tile(plane, x, y)).collect();
        let (lo, hi) = tiles.iter().fold((i32::MAX, i32::MIN), |(lo, hi), t| (lo.min(t.height), hi.max(t.height)));
        let count = |f: &dyn Fn(&crate::cache::map_terrain::Tile) -> bool| tiles.iter().filter(|t| f(t)).count();
        println!(
            "  plane {plane}: heights {lo}..{hi}, {} underlays, {} overlays, {} blocked",
            count(&|t| t.underlay != 0),
            count(&|t| t.overlay != 0),
            count(&|t| t.settings & 1 != 0)
        );
    }
    // Floor colours: underlays by RGB, overlays by packed HSL through the client's colour table.
    use crate::cache::floors::{Overlay, Underlay};
    use crate::client::source::Source;
    let underlays = cache.files(2, 1).unwrap_or_default();
    let overlays = cache.files(2, 4).unwrap_or_default();
    let brightness = crate::client::palette::random_brightness();
    let floor = |files: &crate::client::source::Files, id: u8| files.get(id as usize - 1).cloned().flatten();
    let mut img = image::RgbImage::new(SIZE as u32, SIZE as u32);
    for x in 0..SIZE {
        for y in 0..SIZE {
            let tile = t.tile(0, x, y);
            let shade = |c: i32| -> [u8; 3] {
                let k = (1.0 - tile.height as f64 / -4000.0).clamp(0.5, 1.0);
                [(((c >> 16) & 0xff) as f64 * k) as u8, (((c >> 8) & 0xff) as f64 * k) as u8, ((c & 0xff) as f64 * k) as u8]
            };
            let overlay = (tile.overlay != 0)
                .then(|| floor(&overlays, tile.overlay).and_then(|d| Overlay::decode(&d, tile.overlay as u32 - 1).ok()))
                .flatten()
                .filter(|o| o.colour >= 0);
            let underlay = (tile.underlay != 0).then(|| floor(&underlays, tile.underlay).and_then(|d| Underlay::decode(&d).ok())).flatten();
            let px = match (overlay, underlay) {
                (Some(o), _) => shade(crate::client::palette::hsl_to_rgb(o.colour as u16, brightness)),
                (None, Some(u)) => shade(u.rgb),
                (None, None) => [0, 0, 0],
            };
            // North (higher y) at the top.
            img.put_pixel(x as u32, (SIZE - 1 - y) as u32, image::Rgb(px));
        }
    }
    std::fs::create_dir_all("shots").map_err(|e| e.to_string())?;
    let path = format!("shots/map-{region}.png");
    img.save(&path).map_err(|e| e.to_string())?;
    println!("saved {path}");
    Ok(())
}
