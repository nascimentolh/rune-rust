//! Developer tools that read the local cache directly (no server or window).
//!
//! - `--dump-interface <id>`: every component of an interface with its kind, parent, box,
//!   content and script hooks — the starting point for porting a screen.
//! - `--dump-script <id>`: a CS2 script's instructions.
//! - `--render-interface <id>`: opens an interface as the root window from the local cache,
//!   runs its on-load scripts and saves the drawing to `shots/interface-<id>.png`. With
//!   `RENDER_INPUT="click 382 298; type name; key 84"` it first plays those clicks, typed text and
//!   client key codes through the interface's input hooks, one tick each.
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
        "#{:<3} parent {:<4} {}{} at ({},{}) size {}x{} modes w{} h{} x{} y{} | {content} | onload {:?} | hooks {:?}",
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
        hooks
    )
}

pub fn render_interface(settings: &Settings, id: u32) -> Result<(), String> {
    use crate::client::interface::assets::Assets;
    use crate::client::source::Archives;
    use crate::client::window::Window;
    let cache = Cache::open(&settings.cache_dir).map_err(|e| e.to_string())?;
    let mut window = Window::open(Assets::new(Archives::new(Box::new(cache))), id)?;
    for step in std::env::var("RENDER_INPUT").unwrap_or_default().split(';').map(str::trim).filter(|s| !s.is_empty()) {
        window.ui.play(step)?;
    }
    if let Some((name, password)) = &window.ui.login_request {
        println!("the login window asked to log in as {name:?} with a {}-character password", password.chars().count());
    }
    window.render();
    let path = std::path::PathBuf::from(format!("shots/interface-{id}.png"));
    std::fs::create_dir_all("shots").map_err(|e| e.to_string())?;
    window.raster.save_png(&path)?;
    let lit = window.raster.pixels.iter().filter(|&&p| p != 0).count();
    println!("saved {} ({lit} lit pixels)", path.display());
    Ok(())
}
