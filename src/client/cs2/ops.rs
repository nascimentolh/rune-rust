//! The instructions the login screen's scripts use, from `CS2Interpreter.executeOperation`
//! and the helpers it calls. Everything else stops the script with "not ported".

use super::{Machine, Step};
use crate::cache::cs2::{Instruction, Operand};
use crate::cache::interface::Kind;
use crate::client::interface::component::{Comp, Value};
use crate::client::interface::{CompRef, Ui};

/// Which component an instruction works on: one popped from the stack (`IF_*`, by full id)
/// or the active one (`CC_*`, `hookedInterface1` / `2`).
#[derive(Clone, Copy)]
enum Target {
    Popped,
    Active,
}

/// The component instructions: their target and what they do.
fn component_action(op: &str) -> Option<(Target, &'static str)> {
    use Target::{Active, Popped};
    Some(match op {
        "IF_SETCOLOR" => (Popped, "colour"),
        "CC_SETCOLOR" => (Active, "colour"),
        "IF_SETGRAPHIC" => (Popped, "sprite"),
        "CC_SETGRAPHIC" => (Active, "sprite"),
        "IF_SETTEXT" => (Popped, "text"),
        "CC_SETTEXT" => (Active, "text"),
        "IF_SETPOSITION" => (Popped, "position"),
        "CC_SETPOSITION" => (Active, "position"),
        "IF_SETSIZE" => (Popped, "size"),
        "CC_SETSIZE" => (Active, "size"),
        "CC_SETFILL" => (Active, "fill"),
        "CC_SETTEXTALIGN" => (Active, "align"),
        "CC_SETTEXTFONT" => (Active, "font"),
        "CC_SETTEXTSHADOW" => (Active, "shadow"),
        "CC_SETTILING" => (Active, "tiling"),
        "IF_SETHIDE" => (Popped, "hide"),
        "IF_SETTRANS" => (Popped, "transparency"),
        "IF_SETSCROLLPOS" => (Popped, "scroll"),
        "IF_SETSCROLLSIZE" => (Popped, "scroll size"),
        "IF_SETNOCLICKTHROUGH" => (Popped, "no click-through"),
        "instr6183" => (Popped, "v-flip"),
        "instr6184" => (Popped, "h-flip"),
        "instr6213" => (Popped, "option"),
        "instr6214" => (Popped, "link"),
        "instr6676" => (Active, "link"),
        "instr6073" => (Active, "drag"),
        "IF_GETWIDTH" => (Popped, "get width"),
        "instr6124" => (Active, "get width"),
        "IF_GETHEIGHT" => (Popped, "get height"),
        "instr6125" => (Active, "get height"),
        "IF_GETX" => (Popped, "get x"),
        "instr6552" => (Active, "get x"),
        "IF_GETY" => (Popped, "get y"),
        "instr6772" => (Active, "get y"),
        "instr6151" => (Active, "get slot"),
        "IF_GETHIDE" => (Popped, "get hide"),
        "IF_GETTEXT" => (Popped, "get text"),
        "IF_GETSCROLLY" => (Popped, "get scroll y"),
        "instr6276" => (Popped, "get scroll height"),
        "IF_GETSPRITEID" => (Popped, "get sprite"),
        "instr6293" => (Popped, "get child count"),
        "HOOK_MOUSE_ENTER" => (Popped, "hook onMouseHoverScript"),
        "instr6091" => (Active, "hook onMouseHoverScript"),
        "HOOK_MOUSE_EXIT" => (Popped, "hook onMouseLeaveScript"),
        "instr6092" => (Active, "hook onMouseLeaveScript"),
        "HOOK_MOUSE_PRESS" => (Popped, "hook anObjectArray1386"),
        "instr6088" => (Active, "hook anObjectArray1386"),
        "IF_SETONMOUSEOVER" => (Popped, "hook popupScript"),
        "instr6224" => (Active, "hook anObjectArray1451"),
        "instr6239" => (Popped, "hook params"),
        "instr6246" => (Popped, "hook anObjectArray1412"),
        "instr6248" => (Popped, "hook anObjectArray1292"),
        "instr6255" => (Popped, "hook anObjectArray1353"),
        "instr6256" => (Popped, "hook anObjectArray1271"),
        "instr6342" => (Popped, "hook anObjectArray1409"),
        "instr6499" => (Active, "hook anObjectArray1394"),
        "instr5957" => (Active, "hook anObjectArray1389"),
        _ => return None,
    })
}

fn comp(ui: &mut Ui, r: CompRef) -> Result<&mut Comp, String> {
    ui.comp(r).ok_or_else(|| format!("no component {}:{} slot {}", r.id >> 16, r.id & 0xffff, r.slot))
}

fn bool_of(v: i32) -> bool {
    v == 1
}

/// `popIntArrayParam` + `popParamsDynamic`: a hook's argument types (`s` string, `º` long,
/// anything else int; a trailing `Y` int array is popped and dropped), the values, then the
/// script id (-1 clears the hook).
fn pop_hook(m: &mut Machine) -> Result<Option<Vec<Value>>, String> {
    let mut types = m.pop_string()?;
    if types.ends_with('Y') {
        let n = m.pop_int()?;
        if n > 0 {
            for _ in 0..n {
                m.pop_int()?;
            }
            types.pop();
        }
    }
    let mut values = Vec::new();
    for t in types.chars().rev() {
        values.push(match t {
            's' => Value::Text(m.pop_string()?),
            'º' => Value::Long(m.pop_long()?),
            _ => Value::Int(m.pop_int()?),
        });
    }
    let id = m.pop_int()?;
    if id == -1 {
        return Ok(None);
    }
    values.push(Value::Int(id));
    values.reverse();
    Ok(Some(values))
}

/// A component instruction on `r`.
fn apply(m: &mut Machine, ui: &mut Ui, r: CompRef, action: &'static str) -> Step {
    if let Some(field) = action.strip_prefix("hook ") {
        let hook = pop_hook(m)?;
        comp(ui, r)?.set_hook(field, hook);
        return Ok(());
    }
    match action {
        "colour" => comp(ui, r)?.colour = m.pop_int()?,
        "sprite" => comp(ui, r)?.sprite = m.pop_int()?,
        "text" => comp(ui, r)?.text = m.pop_string()?,
        "fill" => comp(ui, r)?.filled = bool_of(m.pop_int()?),
        "font" => comp(ui, r)?.font = m.pop_int()?,
        "shadow" => comp(ui, r)?.text_shadow = bool_of(m.pop_int()?),
        "tiling" => comp(ui, r)?.tiling = bool_of(m.pop_int()?),
        "hide" => comp(ui, r)?.hidden = bool_of(m.pop_int()?),
        "transparency" => comp(ui, r)?.transparency = m.pop_int()?,
        "v-flip" => comp(ui, r)?.v_flip = bool_of(m.pop_int()?),
        "h-flip" => comp(ui, r)?.h_flip = bool_of(m.pop_int()?),
        "align" => {
            let [h, v, line_height] = m.pop_ints()?;
            let c = comp(ui, r)?;
            (c.h_align, c.v_align, c.line_height) = (h, v, line_height);
        }
        // `setComponentPosition`, then `method913` and, for containers, `method483`.
        "position" => {
            let [x, y, x_mode, y_mode] = m.pop_ints()?;
            let c = comp(ui, r)?;
            (c.base_x, c.base_y) = (x, y);
            (c.x_mode, c.y_mode) = (x_mode.clamp(0, 5) as i8, y_mode.clamp(0, 5) as i8);
            relayout(ui, r)?;
        }
        // `method12260`.
        "size" => {
            let [w, h, w_mode, h_mode] = m.pop_ints()?;
            let c = comp(ui, r)?;
            (c.base_width, c.base_height) = (w, h);
            (c.width_mode, c.height_mode) = (w_mode.clamp(0, 4) as i8, h_mode.clamp(0, 4) as i8);
            relayout(ui, r)?;
        }
        // `method1852`.
        "scroll" => {
            let [x, y] = m.pop_ints()?;
            let c = comp(ui, r)?;
            c.scroll_x = x.min(c.scroll_width - c.width).max(0);
            c.scroll_y = y.min(c.scroll_height - c.height).max(0);
        }
        // `setScrollSize`.
        "scroll size" => {
            let [w, h] = m.pop_ints()?;
            let c = comp(ui, r)?;
            (c.scroll_width, c.scroll_height) = (w, h);
            if c.kind == Kind::Container {
                ui.layout_contents(r);
            }
        }
        // Mouse behaviour and option texts: kept by the reference for clicks, which the
        // login screen doesn't handle yet. Popped exactly as the reference pops them.
        "no click-through" | "drag" => {
            comp(ui, r)?;
            m.pop_int()?;
        }
        "link" => {
            comp(ui, r)?;
            m.pop_ints::<2>()?;
        }
        "option" => {
            comp(ui, r)?;
            m.pop_int()?;
            m.pop_string()?;
        }
        "get width" => {
            let v = comp(ui, r)?.width;
            m.push(v);
        }
        "get height" => {
            let v = comp(ui, r)?.height;
            m.push(v);
        }
        "get x" => {
            let v = comp(ui, r)?.x;
            m.push(v);
        }
        "get y" => {
            let v = comp(ui, r)?.y;
            m.push(v);
        }
        "get slot" => {
            let v = comp(ui, r)?.slot;
            m.push(v);
        }
        "get hide" => {
            let v = comp(ui, r)?.hidden;
            m.push_bool(v);
        }
        "get text" => {
            let v = comp(ui, r)?.text.clone();
            m.string_stack.push(v);
        }
        "get scroll y" => {
            let v = comp(ui, r)?.scroll_y;
            m.push(v);
        }
        "get scroll height" => {
            let v = comp(ui, r)?.scroll_height;
            m.push(v);
        }
        "get sprite" => {
            let v = comp(ui, r)?.sprite;
            m.push(v);
        }
        // `instr6293`: created children up to the first gap.
        "get child count" => {
            let c = comp(ui, r)?;
            let n = c.children.iter().position(Option::is_none).unwrap_or(c.children.len());
            m.push(n as i32);
        }
        other => return Err(format!("unknown component action {other}")),
    }
    Ok(())
}

/// After a position or size change: `method913`, then `method483` for a container.
fn relayout(ui: &mut Ui, r: CompRef) -> Step {
    ui.relayout(r);
    if comp(ui, r)?.kind == Kind::Container {
        ui.layout_contents(r);
    }
    Ok(())
}

/// `method7557` for unaccented text: case-insensitive order first, then length, then case.
///
/// ponytail: accented letters are compared by code point; the reference expands them
/// (`method7778`) and ranks them (`method13422`).
fn compare(a: &str, b: &str) -> i32 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    for (x, y) in a.iter().zip(&b) {
        let (lx, ly) = (x.to_ascii_lowercase(), y.to_ascii_lowercase());
        if lx != ly {
            return lx as i32 - ly as i32;
        }
    }
    if a.len() != b.len() {
        return if a.len() < b.len() { -1 } else { 1 };
    }
    a.iter().zip(&b).find(|(x, y)| x != y).map_or(0, |(x, y)| *x as i32 - *y as i32)
}

/// `(char) v`: the low 16 bits as a character (unpaired surrogates become U+FFFD).
fn java_char(v: i32) -> char {
    char::from_u32(v as u32 & 0xffff).unwrap_or(char::REPLACEMENT_CHARACTER)
}

fn local(locals: &[i32], i: i32) -> Result<i32, String> {
    locals.get(i as usize).copied().ok_or_else(|| format!("no local {i}"))
}

/// One instruction (`CS2Interpreter.executeOperation`).
pub fn step(m: &mut Machine, ui: &mut Ui, ins: &Instruction) -> Step {
    let op = ins.name();
    let operand = ins.int();
    if let Some((target, action)) = component_action(op) {
        let r = match target {
            Target::Popped => CompRef::of(m.pop_int()? as u32),
            Target::Active => ui.active[usize::from(m.second)].ok_or("no active component")?,
        };
        return apply(m, ui, r, action);
    }
    match op {
        "PUSH_INT" => m.push(operand),
        "PUSH_STRING" => {
            let Operand::Text(t) = &ins.operand else { return Err("no string operand".into()) };
            m.string_stack.push(t.clone());
        }
        "LOAD_INT" => {
            let v = local(&m.int_locals, operand)?;
            m.push(v);
        }
        "STORE_INT" => {
            let v = m.pop_int()?;
            *m.int_locals.get_mut(operand as usize).ok_or("no such local")? = v;
        }
        "LOAD_STRING" => {
            let v = m.string_locals.get(operand as usize).ok_or("no such local")?.clone();
            m.string_stack.push(v);
        }
        "STORE_STRING" => {
            let v = m.pop_string()?;
            *m.string_locals.get_mut(operand as usize).ok_or("no such local")? = v;
        }
        "LOAD_VARC" => m.push(ui.varc.get(&operand).copied().unwrap_or(0)),
        "STORE_VARC" => {
            let v = m.pop_int()?;
            ui.varc.insert(operand, v);
        }
        "LOAD_VARC_STRING" => m.string_stack.push(ui.varc_strings.get(&operand).cloned().unwrap_or_default()),
        "STORE_VARC_STRING" => {
            let v = m.pop_string()?;
            ui.varc_strings.insert(operand, v);
        }
        "POP_INT" => {
            m.pop_int()?;
        }
        "POP_2_INT" => {
            m.pop_ints::<2>()?;
        }
        "GOTO" => m.pc += operand as isize,
        "INT_EQ" | "INT_NE" | "INT_LT" | "INT_GT" | "INT_LE" | "INT_GE" => {
            let [a, b] = m.pop_ints()?;
            let jump = match op {
                "INT_EQ" => a == b,
                "INT_NE" => a != b,
                "INT_LT" => a < b,
                "INT_GT" => a > b,
                "INT_LE" => a <= b,
                _ => a >= b,
            };
            if jump {
                m.pc += operand as isize;
            }
        }
        "SWITCH" => {
            let v = m.pop_int()?;
            let table = m.script.switches.get(operand as usize).ok_or("no such switch")?;
            if let Some(&jump) = table.get(&v) {
                m.pc += jump as isize;
            }
        }
        "CALL_CS2" => m.call(ui, operand)?,
        "RETURN" => m.ret(),
        "ADD" | "SUBTRACT" | "MULTIPLY" | "DIVIDE" | "MODULO" | "MIN" | "MAX" | "BIT_FLAGGED" | "SET_BIT" => {
            let [a, b] = m.pop_ints()?;
            let zero = || -> Result<(), String> {
                if b == 0 {
                    Err("division by zero".into())
                } else {
                    Ok(())
                }
            };
            let v = match op {
                "ADD" => a.wrapping_add(b),
                "SUBTRACT" => a.wrapping_sub(b),
                "MULTIPLY" => a.wrapping_mul(b),
                "DIVIDE" => {
                    zero()?;
                    a.wrapping_div(b)
                }
                "MODULO" => {
                    zero()?;
                    a.wrapping_rem(b)
                }
                "MIN" => a.min(b),
                "MAX" => a.max(b),
                "BIT_FLAGGED" => i32::from(a & 1i32.wrapping_shl(b as u32) != 0),
                _ => a | 1i32.wrapping_shl(b as u32),
            };
            m.push(v);
        }
        "SCALE" => {
            let [a, b, c] = m.pop_ints()?;
            if b == 0 {
                return Err("division by zero".into());
            }
            m.push((c as i64 * a as i64 / b as i64) as i32);
        }
        "RANDOM" => {
            let max = m.pop_int()?;
            m.push((rand::random::<f64>() * max as f64) as i32);
        }
        "APPEND" => {
            let b = m.pop_string()?;
            let a = m.pop_string()?;
            m.string_stack.push(a + &b);
        }
        "MERGE_STRINGS" => {
            let at = m.string_stack.len().checked_sub(operand as usize).ok_or("string stack empty")?;
            let merged = m.string_stack.split_off(at).concat();
            m.string_stack.push(merged);
        }
        "TOSTRING" => {
            let v = m.pop_int()?;
            m.string_stack.push(v.to_string());
        }
        "STRING_LENGTH" => {
            let s = m.pop_string()?;
            m.push(s.chars().count() as i32);
        }
        "SUBSTRING" => {
            let s = m.pop_string()?;
            let [from, to] = m.pop_ints()?;
            let chars: Vec<char> = s.chars().collect();
            let part = usize::try_from(from)
                .ok()
                .zip(usize::try_from(to).ok())
                .filter(|(f, t)| f <= t && *t <= chars.len())
                .map(|(f, t)| chars[f..t].iter().collect::<String>())
                .ok_or("substring out of range")?;
            m.string_stack.push(part);
        }
        "STRING_INDEXOF_STRING" => {
            let needle = m.pop_string()?;
            let hay = m.pop_string()?;
            let from = m.pop_int()?.max(0) as usize;
            let (hay, needle): (Vec<char>, Vec<char>) = (hay.chars().collect(), needle.chars().collect());
            let found = (from..=hay.len()).find(|&i| hay[i..].starts_with(&needle));
            m.push(found.map_or(-1, |i| i as i32));
        }
        // `method6117`: -1 (no character) is an error.
        "APPEND_CHAR" => {
            let text = m.pop_string()?;
            let c = m.pop_int()?;
            if c == -1 {
                return Err("no character".into());
            }
            m.string_stack.push(format!("{text}{}", java_char(c)));
        }
        "CHAR_ISPRINTABLE" | "CHAR_ISALPHANUMERIC" | "CHAR_ISALPHA" | "CHAR_ISNUMERIC" => {
            let c = (m.pop_int()? as u32) & 0xffff;
            let yes = match op {
                // `Node_Sub25.method12402`.
                "CHAR_ISPRINTABLE" => (32..=126).contains(&c) || (160..=255).contains(&c) || [8364, 338, 8212, 339, 376].contains(&c),
                // `Class380.method6450`, `Class60.method1169`, `AnimationIndexLoader.method11219`.
                "CHAR_ISALPHANUMERIC" => matches!(c, 48..=57 | 65..=90 | 97..=122),
                "CHAR_ISALPHA" => matches!(c, 65..=90 | 97..=122),
                _ => (48..=57).contains(&c),
            };
            m.push_bool(yes);
        }
        // `Character.toLowerCase`.
        "CHAR_TOLOWERCASE" => {
            let c = java_char(m.pop_int()?);
            m.push(c.to_lowercase().next().unwrap_or(c) as i32);
        }
        // `removeImageTags`: drops everything from `<` to `>`.
        "REMOVETAGS" => {
            let text = m.pop_string()?;
            let mut inside = false;
            let plain: String = text
                .chars()
                .filter(|&c| {
                    match c {
                        '<' => inside = true,
                        '>' => inside = false,
                        _ => return !inside,
                    }
                    false
                })
                .collect();
            m.string_stack.push(plain);
        }
        "COMPARE" => {
            let b = m.pop_string()?;
            let a = m.pop_string()?;
            m.push(compare(&a, &b));
        }
        "STRINGWIDTH" => {
            let text = m.pop_string()?;
            let font = m.pop_int()?;
            let w = ui.assets.metrics(font).ok_or(format!("no font {font}"))?.text_width(&text);
            m.push(w);
        }
        // `method6949`: how many lines the text wraps to; `method6951`: the widest line.
        "PARAHEIGHT" | "PARAWIDTH" => {
            let text = m.pop_string()?;
            let [width, font] = m.pop_ints()?;
            let metrics = ui.assets.metrics(font).ok_or(format!("no font {font}"))?;
            let lines = metrics.split_lines(&text, Some(width));
            let v = if op == "PARAHEIGHT" { lines.len() as i32 } else { lines.iter().map(|l| metrics.text_width(l)).max().unwrap_or(0) };
            m.push(v);
        }
        "CLIENTCLOCK" => m.push(ui.clock()),
        // The window has focus while the login screen is in front.
        "APPLET_HASFOCUS" => m.push(1),
        // `javaBuild >= 6u10`: this client isn't a Java applet, so the check always passes.
        "CHECK_JAVA_VERSION" => m.push(1),
        // `Language.ENGLISH.getValue()`.
        "WORLD_LANGUAGE" => m.push(0),
        "CC_CREATE" => {
            let [parent, kind, slot] = m.pop_ints()?;
            create(m, ui, parent as u32, kind, slot)?;
        }
        "CC_DELETE" => {
            let r = ui.active[usize::from(m.second)].ok_or("no active component")?;
            if r.slot == -1 {
                return Err("not a created component".into());
            }
            let parent = comp(ui, CompRef::of(r.id))?;
            *parent.children.get_mut(r.slot as usize).ok_or("no such slot")? = None;
        }
        "CC_DELETEALL" => {
            let id = m.pop_int()? as u32;
            comp(ui, CompRef::of(id))?.children.clear();
        }
        "CC_SETCHILD_SLOT" => {
            let [id, slot] = m.pop_ints()?;
            let found = slot != -1 && {
                let r = CompRef { id: id as u32, slot };
                let ok = ui.comp(r).is_some();
                ui.active[usize::from(m.second)] = ok.then_some(r);
                ok
            };
            m.push_bool(found);
        }
        "IF_OPENSUBCLIENT" => {
            let [parent, interface] = m.pop_ints()?;
            ui.open_sub(parent as u32, interface as u32);
        }
        "IF_CLOSESUBCLIENT" => {
            let parent = m.pop_int()? as u32;
            ui.close_sub(parent, true);
        }
        "SETUP_MESSAGEBOX" => setup_message_box(m)?,
        // `Class155.method2635`: log in with name and password (names up to 320 characters,
        // only from the login screen with no login under way). `Class455.method7558` then sets
        // the reply code at once: -3 (connecting), or 3 when the name or password is empty.
        "instr6707" => {
            let password = m.pop_string()?;
            let name = m.pop_string()?;
            if name.chars().count() <= 320 && ui.login_request.is_none() && ui.login_reply != -3 {
                if name.is_empty() || password.is_empty() {
                    ui.login_reply = 3;
                } else {
                    ui.login_reply = -3;
                    ui.login_request = Some((name, password));
                }
            }
        }
        // `getEnumValue`: the asked key and value types must match the enum's.
        "ENUM" => {
            let [key_type, value_type, id, key] = m.pop_ints()?;
            let e = ui.assets.enumeration(id)?;
            if key_type != e.key_type as i32 || value_type != e.value_type as i32 {
                return Err(format!("enum {id} {key}"));
            }
            if value_type == 's' as i32 {
                m.string_stack.push(e.text(key)?);
            } else {
                m.push(e.int(key)?);
            }
        }
        "STRUCT_PARAM" => {
            let [id, param] = m.pop_ints()?;
            match ui.assets.struct_param(id, param)? {
                crate::cache::enums::EnumValue::Text(t) => m.string_stack.push(t),
                crate::cache::enums::EnumValue::Int(v) => m.push(v),
            }
        }
        "ENUM_GETOUTPUTCOUNT" => {
            let id = m.pop_int()?;
            let size = ui.assets.enumeration(id)?.size;
            m.push(size);
        }
        _ => match settings(op, m, ui)? {
            Some(v) => m.push(v),
            None if handled_without_value(op, m, ui)? => {}
            None => return Err("not ported".into()),
        },
    }
    Ok(())
}

/// `SETUP_MESSAGEBOX` (`Node_Sub15_Sub3.method15239`): the style of the login screen's
/// message box: two enum choices (`Class356`, `Class353`, three values each), then nine ints.
///
/// The box isn't drawn yet, so the style is checked and dropped.
fn setup_message_box(m: &mut Machine) -> Step {
    let [style, align, ..] = m.pop_ints::<11>()?;
    if !(0..3).contains(&style) || !(0..3).contains(&align) {
        return Err("no such message box style".into());
    }
    Ok(())
}

/// Client parameter 26 (`Loader.java`).
const CLIENT_PARAM_26: i64 = 637809750146160229;

/// Instructions that push one value from the preferences or the client's state.
fn settings(op: &str, m: &mut Machine, ui: &mut Ui) -> Result<Option<i32>, String> {
    use crate::client::preferences::toolkit_has_effects;
    let p = &ui.prefs;
    let is = |v: i32, want: i32| i32::from(v == want);
    Ok(Some(match op {
        "instr5954" => is(p.ground_blending, 1),
        "instr6060" => is(p.fog, 1),
        "instr6285" => is(p.remove_roofs, 2),
        "instr6286" => is(p.textures, 1),
        "instr6489" => is(p.flickering, 1),
        "instr6640" => is(p.bloom, 1),
        "instr6755" => is(p.mono_stereo, 1),
        "instr6756" => is(p.ground_decor, 1),
        "instr6759" => is(p.character_shadows, 1),
        "instr6761" => is(p.light_detail, 1),
        "DETAILGET_WATERDETAIL_HIGH" => is(p.water, 2),
        "instr6301" => p.voice_volume,
        "instr6766" => p.sound_volume,
        "instr6767" => p.music_volume,
        "instr6768" => p.ambient_volume,
        "instr6783" => p.sub13_volume,
        "instr6406" => p.screen_size,
        "instr6600" => p.current_toolkit,
        "instr6754" => p.brightness,
        "instr6777" => p.toolkit,
        "instr6778" => p.cpu,
        "instr6781" => p.max_screen_size,
        "instr6785" => p.scenery_shadows,
        "instr6789" => p.skyboxes,
        "instr6859" => p.graphics,
        "instr6925" => p.idle_animations,
        "instr6938" => p.anti_aliasing,
        "DETAILCANMOD_SKYBOXES" => i32::from(toolkit_has_effects(p.current_toolkit)),
        "DETAILCANMOD_TOOLKIT" => 1,
        // `GraphicsToolkitPreference.checkValid`: toolkit 3 needs the DirectX library.
        "DETAILCANSET_RENDERER" => {
            let toolkit = m.pop_int()?;
            if toolkit == 3 {
                3
            } else {
                2
            }
        }
        "PROFILE_CPU" => crate::client::preferences::cpu_profile(),
        // `Class9.anInt72`: how the login from this screen went (see `Ui::login_reply`).
        "instr6709" => ui.login_reply,
        // `aByteArray7152`: client parameter 1 ("", shorter than 16 bytes, so null).
        "instr6710" => 0,
        // `aString7165` (client parameter 3, not given): pushes "".
        "instr6721" => {
            m.string_stack.push(String::new());
            return Ok(None);
        }
        // `aLong7153`: client parameter 26, 637809750146160229, as high then low int.
        "instr6711" => {
            m.push((CLIENT_PARAM_26 >> 32) as i32);
            CLIENT_PARAM_26 as i32
        }
        // `aBool7171`: set when toolkit 2 couldn't start (see `preferences`).
        // `instr6852` (Auto Setup): pushes the current toolkit, then the detail level chosen.
        "instr6852" => {
            let level = ui.prefs.auto_setup();
            m.push(ui.prefs.current_toolkit);
            level
        }
        "instr6775" => 1,
        // `aBool7465`: safe mode was on at start-up (default off).
        "instr6784" => 0,
        // `aBool7151`: client parameter 29, "false".
        "FROMBILLING" => 0,
        // `aBool7155`: "s" held down at start-up for safe mode.
        "instr6097" => 0,
        // Full-screen modes (`Class475.supportsFullScreen` is false for this window):
        // the count, and the index of the current one.
        "instr6704" => 0,
        // `Class158.windowedMode`: 1 fixed size (2 resizable, 3 full screen).
        "instr6644" => 1,
        "instr6643" => -1,
        _ => return Ok(None),
    }))
}

/// Instructions with no value to push: settings changes and the 3D scene behind the
/// login screen, which isn't drawn yet (its arguments are popped as the reference pops them).
fn handled_without_value(op: &str, m: &mut Machine, ui: &mut Ui) -> Result<bool, String> {
    match op {
        // `instr6647`: screen size 1 or 2 (also `aPreference_Sub9_8218`), then saved.
        "instr6647" => {
            let v = m.pop_int()?;
            if (1..=2).contains(&v) {
                ui.prefs.screen_size = v;
            }
        }
        // `instr6629`: the login music volume (`aPreference_Sub13_8229`). The reference also
        // starts, stops or re-levels the login music here; music isn't played yet.
        "instr6629" => {
            ui.prefs.sub13_volume = m.pop_int()?;
        }
        // The Options panel's detail buttons (`MaterialProp14.method15393` … `EquipmentDefaults
        // .method11248`), then saved.
        "AUTOSETUP_SETMIN" | "AUTOSETUP_SETLOW" | "AUTOSETUP_SETMEDIUM" | "AUTOSETUP_SETHIGH" => {
            let level = ["AUTOSETUP_SETMIN", "AUTOSETUP_SETLOW", "AUTOSETUP_SETMEDIUM", "AUTOSETUP_SETHIGH"]
                .iter()
                .position(|&o| o == op)
                .unwrap_or(0);
            ui.prefs.apply_preset(level as i32 + 1);
        }
        // Volume sliders (`SoundLevelPreference` takes any value). Music volume also starts or
        // stops the in-game music once logged in; music isn't played yet.
        "DETAIL_MUSICVOL" => ui.prefs.music_volume = m.pop_int()?,
        "DETAIL_SOUNDVOL" => ui.prefs.sound_volume = m.pop_int()?,
        "DETAIL_BGSOUNDVOL" => ui.prefs.ambient_volume = m.pop_int()?,
        "instr6750" => ui.prefs.voice_volume = m.pop_int()?,
        // `method3942`: 1 = stereo, anything else mono.
        "DETAIL_STEREO" => ui.prefs.mono_stereo = i32::from(m.pop_int()? == 1),
        // `instr6922`: the saved toolkit (refused for 3, DirectX, which isn't there), and the
        // graphics preference back to 0 unless the second value is 1; then saved.
        "instr6922" => {
            let [toolkit, keep_graphics] = m.pop_ints()?;
            if toolkit != 3 {
                ui.prefs.toolkit = toolkit;
            }
            if keep_graphics != 1 {
                ui.prefs.graphics = 0;
            }
        }
        // `instr6752`: `aPreference_Sub11_8217`, 0 when out of 0..=255.
        "instr6752" => {
            let v = m.pop_int()?;
            ui.prefs.sub11 = if (0..=255).contains(&v) { v } else { 0 };
        }
        // Preferences the login screen sets but never reads back here.
        "DETAIL_CUSTOMCURSOR" | "DETAIL_REMOVEROOFS_OPTION_OVERRIDE" | "instr6645" => {
            m.pop_int()?;
        }
        // Full-screen mode sizes: none supported.
        "instr6404" => {
            m.pop_int()?;
            m.push(0);
            m.push(0);
        }
        // `Class361.method6257`: calls a JavaScript function of the applet's web page
        // (`_8`, `_15`), ignoring failures. There's no page here.
        "instr6410" | "instr6680" => {}
        // `instr6694`: when no login is under way (`loginStage` 2), resets the login reply
        // codes to -2, which they already are here.
        "instr6694" => {}
        // The camera path and camera of the 3D login scene.
        "instr6650" | "instr6666" => {
            m.pop_ints::<2>()?;
        }
        "instr6162" | "instr6675" => {
            m.pop_ints::<4>()?;
        }
        "instr6651" => {
            m.pop_ints::<7>()?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `CC_CREATE` (`method2738`): a new child in the parent's slot list, made active.
fn create(m: &Machine, ui: &mut Ui, parent: u32, kind: i32, slot: i32) -> Step {
    if kind == 0 {
        return Err("can't create a container".into());
    }
    let index = usize::try_from(slot).map_err(|_| "negative slot")?;
    let p = comp(ui, CompRef::of(parent))?;
    if p.children.len() <= index {
        p.children.resize(index + 1, None);
    }
    if index > 0 && p.children[index - 1].is_none() {
        return Err(format!("slot {} is empty", index - 1));
    }
    let id = p.id;
    p.children[index] = Some(Comp::created(id, Kind::from_id(kind as u8), slot));
    ui.active[usize::from(m.second)] = Some(CompRef { id, slot });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_like_the_client() {
        assert_eq!(compare("abc", "ABC"), 'a' as i32 - 'A' as i32);
        assert_eq!(compare("ab", "abc"), -1);
        assert!(compare("b", "a") > 0);
        assert_eq!(compare("same", "same"), 0);
    }
}
