---
name: port-cs2-instruction
description: Make an interface's CS2 scripts run further — find the first unported instruction with the project's tools, read its Java in CS2Interpreter, port it into src/client/cs2/ops.rs and repeat. Use when a screen is incomplete or a script prints "not ported".
---

# Port a CS2 instruction

1. Run the interface offline (needs `server.cfg` with your build-727 cache):
   `cargo run -- --render-interface 744` — add `RENDER_INPUT="click x y; type text; key code; reply code"`
   to replay clicks and typing. It prints lines like
   `script 1172 stopped: SETUP_MESSAGEBOX (opcode 211) at 11 of script 2692: not ported`
   and saves `shots/interface-<id>.png`.
2. In the reference, find the case: `grep -n "case SETUP_MESSAGEBOX:" -A2 CS2Interpreter.java`,
   then the helper it calls and every helper below it.
3. Port it in `src/client/cs2/ops.rs`:
   - component instructions (`IF_*` pop the component id first, `CC_*` use the active one) go in
     `component_action` / `apply`;
   - values from preferences or client state go in `settings`;
   - effects with nothing to push go in `handled_without_value`;
   - anything else in the main `match` of `step`.
   Pop in the reference's order; an error ends the script like the reference's exception.
4. When the instruction depends on client state (login codes, preferences, server data), port
   that state too and find every place the reference changes it.
5. Rerun step 1 until nothing stops, and look at the picture. Useful helpers:
   `--scripts-using N` (who calls script N / uses component N), `--dump-script N`,
   `--script-scope N`.
6. Finish with `cite-reference` and `check-and-commit`.
