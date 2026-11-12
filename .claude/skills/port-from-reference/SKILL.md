---
name: port-from-reference
description: Port a behaviour of the 727 Java reference client to Rune Rust faithfully — find the Java, read every line it calls, port it into the right module, cite it and leave a check. Use for any new feature, screen, decoder or server rule.
---

# Port from the reference

Follow `docs/PORTING.md`. In short:

1. **Locate the reference.** Ask the user where the reference client is if you don't know
   (`…/src/main/java/com/jagex`). Read files as Latin-1: `iconv -f latin1 -t utf-8 File.java`.
2. **Find the code.** Start from something visible (a string, a CS2 instruction name, a field
   the feature sets) and `grep -rn` until you reach the function that does it. Follow every call,
   and grep every assignment to each static field it reads (`grep -rn "Class9.anInt72 = "`).
3. **Read all of it before writing.** Note values, orders, signed/unsigned arithmetic, wrap-around
   and quirks. If a part can't be found, stop and say so; never fill it with a guess.
4. **Check what comes from the server.** If the client reacts to server data, the Rust server
   must send it the way the reference expects. Say so when that side isn't done.
5. **Port into the right module** (table in `docs/PORTING.md`). Readable names; the reference
   name in the doc comment. Keep Java semantics (`as i8`, `wrapping_*`, `>>` vs `>>>`).
6. **Leave one check**: a unit test for a decoder/formula, or a self-test / tool run for a screen.
7. **Cite it** with the `cite-reference` skill, then finish with `check-and-commit`.
