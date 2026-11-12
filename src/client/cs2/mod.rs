//! The CS2 script machine. Port of `CS2Executor` (`executeHookInner`, `executeScript`)
//! with the instructions of `CS2Interpreter` in `ops`.
//!
//! As in the reference, any failure (a bad stack, a missing component, an instruction
//! not ported yet) ends the whole script; the reason is printed once per kind.

mod calendar;
mod ops;

use crate::cache::cs2::Script;
use crate::client::interface::component::Value;
use crate::client::interface::{CompRef, Ui};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Mutex;

/// A script run with its arguments (`HookRequest`): script id, then int/string/long values.
#[derive(Clone, Debug, Default)]
pub struct Hook {
    pub source: Option<CompRef>,
    pub args: Vec<Value>,
    /// `mouseX` / `mouseY`, relative to the source component.
    pub mouse: (i32, i32),
    /// `typedKeyCode` / `typedKeyChar`.
    pub key: (i32, i32),
    /// `anInt8051`: the option (1..=10) a menu click picked, 0 otherwise.
    pub op: i32,
}

/// The magic hook arguments filled in when the hook runs (`executeHookInner`).
const MOUSE_X: i32 = -2147483647;
const MOUSE_Y: i32 = -2147483646;
const SOURCE_ID: i32 = -2147483645;
/// `anInt8051`: the option a menu click picked.
const OP_INDEX: i32 = -2147483644;
const SOURCE_SLOT: i32 = -2147483643;
/// `aClass118_8057`, the drag target, which no hook here has: -1.
const TARGET_ID: i32 = -2147483642;
const TARGET_SLOT: i32 = -2147483641;
const KEY_CODE: i32 = -2147483640;
const KEY_CHAR: i32 = -2147483639;

/// `returnValues`: the caller's place, kept by `CALL_CS2` and restored by `RETURN`.
struct Frame {
    id: i32,
    script: Rc<Script>,
    pc: isize,
    ints: Vec<i32>,
    strings: Vec<String>,
    longs: Vec<i64>,
}

pub struct Machine {
    int_stack: Vec<i32>,
    string_stack: Vec<String>,
    long_stack: Vec<i64>,
    frames: Vec<Frame>,
    /// The running script's id (`current.pointer`), for error reports.
    id: i32,
    script: Rc<Script>,
    pc: isize,
    int_locals: Vec<i32>,
    string_locals: Vec<String>,
    long_locals: Vec<i64>,
    /// `aBool7022`: the instruction's operand is 1, so `CC_*` use the second active component.
    second: bool,
}

pub type Step = Result<(), String>;

/// `executeHookInner(hook, limit)`.
pub fn run_hook(ui: &mut Ui, hook: &Hook, limit: u32) {
    let Some(Value::Int(id)) = hook.args.first() else { return };
    let Some(script) = ui.assets.script(*id) else { return };
    let mut m = Machine {
        int_stack: Vec::new(),
        string_stack: Vec::new(),
        long_stack: Vec::new(),
        frames: Vec::new(),
        int_locals: vec![0; script.int_locals as usize],
        string_locals: vec![String::new(); script.string_locals as usize],
        long_locals: vec![0; script.long_locals as usize],
        id: *id,
        script,
        pc: -1,
        second: false,
    };
    let (mut ints, mut strings, mut longs) = (0, 0, 0);
    for arg in &hook.args[1..] {
        match arg {
            Value::Int(v) => {
                let v = match *v {
                    MOUSE_X => hook.mouse.0,
                    MOUSE_Y => hook.mouse.1,
                    SOURCE_ID => hook.source.map_or(-1, |s| s.id as i32),
                    OP_INDEX => hook.op,
                    SOURCE_SLOT => hook.source.map_or(-1, |s| s.slot),
                    TARGET_ID | TARGET_SLOT => -1,
                    KEY_CODE => hook.key.0,
                    KEY_CHAR => hook.key.1,
                    v => v,
                };
                set(&mut m.int_locals, &mut ints, v);
            }
            Value::Text(t) => set(&mut m.string_locals, &mut strings, t.clone()),
            Value::Long(v) => set(&mut m.long_locals, &mut longs, *v),
        }
    }
    if let Err(e) = m.execute(ui, limit) {
        report(*id, &e);
    }
}

/// Stores the next local of a kind; arguments beyond the script's locals are dropped.
fn set<T>(locals: &mut [T], next: &mut usize, value: T) {
    if let Some(slot) = locals.get_mut(*next) {
        *slot = value;
    }
    *next += 1;
}

/// Prints why a script stopped, once per script and reason.
fn report(script: i32, reason: &str) {
    static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let line = format!("script {script} stopped: {reason}");
    if SEEN.lock().map(|mut s| s.get_or_insert_with(HashSet::new).insert(line.clone())).unwrap_or(true) {
        eprintln!("{line}");
    }
}

impl Machine {
    /// `executeScript`: runs until the outermost `RETURN`, at most `limit` instructions.
    fn execute(&mut self, ui: &mut Ui, limit: u32) -> Step {
        for _ in 0..limit {
            self.pc += 1;
            let script = Rc::clone(&self.script);
            let ins = script.instructions.get(self.pc as usize).ok_or("ran past the last instruction")?;
            self.second = ins.int() == 1;
            if ins.name() == "RETURN" && self.frames.is_empty() {
                return Ok(());
            }
            ops::step(self, ui, ins)
                .map_err(|e| format!("{} (opcode {}) at {} of script {}: {e}", ins.name(), ins.opcode, self.pc, self.id))?;
        }
        Err("instruction limit reached".into())
    }

    fn pop_int(&mut self) -> Result<i32, String> {
        self.int_stack.pop().ok_or_else(|| "int stack empty".into())
    }

    fn pop_ints<const N: usize>(&mut self) -> Result<[i32; N], String> {
        let at = self.int_stack.len().checked_sub(N).ok_or("int stack empty")?;
        let mut out = [0; N];
        out.copy_from_slice(&self.int_stack[at..]);
        self.int_stack.truncate(at);
        Ok(out)
    }

    fn pop_string(&mut self) -> Result<String, String> {
        self.string_stack.pop().ok_or_else(|| "string stack empty".into())
    }

    fn pop_long(&mut self) -> Result<i64, String> {
        self.long_stack.pop().ok_or_else(|| "long stack empty".into())
    }

    fn push(&mut self, v: i32) {
        self.int_stack.push(v);
    }

    fn push_bool(&mut self, b: bool) {
        self.int_stack.push(i32::from(b));
    }

    /// `CALL_CS2`.
    fn call(&mut self, ui: &mut Ui, id: i32) -> Step {
        let callee = ui.assets.script(id).ok_or(format!("script {id} missing"))?;
        if self.frames.len() >= 50 {
            return Err("call stack full".into());
        }
        let ints = self.int_stack.split_off(self.int_stack.len().checked_sub(callee.int_args as usize).ok_or("int stack empty")?);
        let strings =
            self.string_stack.split_off(self.string_stack.len().checked_sub(callee.string_args as usize).ok_or("string stack empty")?);
        let longs = self.long_stack.split_off(self.long_stack.len().checked_sub(callee.long_args as usize).ok_or("long stack empty")?);
        let mut frame = Frame {
            id,
            script: Rc::clone(&callee),
            pc: -1,
            ints: vec![0; callee.int_locals as usize],
            strings: vec![String::new(); callee.string_locals as usize],
            longs: vec![0; callee.long_locals as usize],
        };
        for (slot, v) in frame.ints.iter_mut().zip(ints) {
            *slot = v;
        }
        for (slot, v) in frame.strings.iter_mut().zip(strings) {
            *slot = v;
        }
        for (slot, v) in frame.longs.iter_mut().zip(longs) {
            *slot = v;
        }
        self.swap_frame(&mut frame);
        self.frames.push(frame);
        Ok(())
    }

    /// `RETURN` from a called script.
    fn ret(&mut self) {
        if let Some(mut frame) = self.frames.pop() {
            self.swap_frame(&mut frame);
        }
    }

    fn swap_frame(&mut self, f: &mut Frame) {
        std::mem::swap(&mut self.id, &mut f.id);
        std::mem::swap(&mut self.script, &mut f.script);
        std::mem::swap(&mut self.pc, &mut f.pc);
        std::mem::swap(&mut self.int_locals, &mut f.ints);
        std::mem::swap(&mut self.string_locals, &mut f.strings);
        std::mem::swap(&mut self.long_locals, &mut f.longs);
    }
}
