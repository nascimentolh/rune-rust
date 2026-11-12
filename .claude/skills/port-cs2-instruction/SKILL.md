---
name: port-cs2-instruction
description: Make an interface's CS2 scripts run further — find the first unported instruction with the project's tools, read its Java in CS2Interpreter, port it into src/client/cs2/ops.rs and repeat until the screen matches. Use when a screen is incomplete, a button does nothing, or a script prints "not ported".
---

# Port a CS2 instruction

## The loop

1. `cargo run -- --render-interface 744` (any interface id) runs the interface's on-load scripts
   against your cache, saves `shots/interface-744.png` and prints each failure once:
   `script 1172 stopped: SETUP_MESSAGEBOX (opcode 211) at 11 of script 2692: not ported`.
   The first id is the hook's script; "of script N" is where it actually stopped.
2. Replay input:
   `RENDER_INPUT="click 382 298; type name; key 84; reply 3" cargo run -- --render-interface 744`
   - `click x y` = press, then release on the next tick; `type text` = typed characters
     (code -1); `key n` = a client key code press (table `CanvasKeyRecorder.anIntArray8044`:
     Enter 84, Tab 80, Backspace 85, Escape 13, Space 83); `reply n` = sets the login reply code
     the scripts read, then one tick.
   - Login window spots that work: Leave Alone (382,298), Auto Setup (382,273), login box
     (380,186), password box (380,233), Log In (381,274), Options (722,17), sound (703,17).
3. Find the Java: `grep -n "case SETUP_MESSAGEBOX:" -A2 CS2Interpreter.java`, then the helper
   and all its callees. Note **what it pops, in which order, and what it pushes**.
4. Port it (below), rebuild, rerun step 1 or 2 until nothing stops, and look at the picture.

## Where an instruction goes in `src/client/cs2/ops.rs`

| Kind | Place | Notes |
|---|---|---|
| Sets or gets a component field | `component_action` + `apply` | `IF_*` pop the component id **first**; `CC_*` use the active component (`ui.active[second]`) |
| Sets a hook (`HOOK_*`, `IF_SETON*`, many `instrNNNN`) | `component_action` with `"hook <reference field>"` | `pop_hook` = `popParamsDynamic`: type string; a trailing `Y` int array is popped (the `Y` is only trimmed when its size > 0); values in reverse; then the script id (-1 clears) |
| Pushes a preference or client-state value | `settings` | derive the value from the reference (preference defaults, client params) |
| Changes state, pushes nothing | `handled_without_value` | 3D-scene camera instructions pop their values and are ignored until the scene exists |
| Everything else | the main `match` in `step` | arithmetic must wrap; division by zero is an error |

## CS2 facts that are easy to get wrong

- An instruction whose int operand is 1 uses the **second** active component (`aBool7022`).
- Branches move `pc += operand`; `SWITCH` jumps by the script's switch table.
- Hook arguments are magic ints filled when the hook runs: mouse x/y (-2147483647 / -2147483646),
  source id (-2147483645), op index (-2147483644 → 0), source slot (-2147483643), drag target
  id/slot (-2147483642 / -2147483641 → -1 when none), key code/char (-2147483640 / -2147483639).
  See `src/client/cs2/mod.rs`.
- In a hook's type string, `º` (Latin-1 0xBA) marks a long, `s` a string, anything else an int.
- On-load scripts run with a 2,000,000-instruction limit; input hooks with 200,000.
- `IF_OPENSUBCLIENT` from a script does **not** run the sub-interface's on-load scripts.
- Scripts often poll client state every tick through a hook set with `instr6342`
  (`anObjectArray1409`). If a screen never updates, check that the state they read changes at
  the same moment as in the reference (for example `instr6707` sets the login code to -3 at once).
- Hooks run in three queues per tick: every-tick first, then button-up/leave, then the rest.

## Finding who does what in the cache

- `cargo run -- --scripts-using 2948`: scripts that call script 2948 or push that number
  (component ids are `interface << 16 | index`, e.g. 39059462 = 596:6).
- `cargo run -- --dump-script N`, `--dump-interface N`, `--script-scope N`.

Finish with `cite-reference`, update `ROADMAP.md`, then `check-and-commit`.
