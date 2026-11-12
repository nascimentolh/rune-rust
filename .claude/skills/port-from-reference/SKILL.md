---
name: port-from-reference
description: Port a behaviour of the 727 Java reference client to Rune Rust faithfully — find the Java, read every line it calls, port it into the right module, cite it and leave a check. Use for any new feature, screen, decoder, preference or server rule.
---

# Port from the reference

The overview is `docs/PORTING.md`; what's done and what's next is `ROADMAP.md`. This is the
checklist, with the traps that cost time before.

## 0. Before you start

- You need the reference client sources (`open727Client`, folder `src/main/java/com/jagex`) and a
  **build-727** cache (OpenRS2 #309) with `server.cfg` pointing at it. A build-718 cache loads, but
  its CS2 scripts decode as nonsense: script opcodes are renumbered every build.
- Ask the user for the reference path if you don't have it. Never invent a path or a value.
- Read Java as Latin-1: `iconv -f latin1 -t utf-8 File.java | sed -n 100,200p`. Reading it as
  UTF-8 fails or garbles `º`, which matters: it's a CS2 type letter.
- Check `ROADMAP.md` first: the feature may be partly done, with notes on where it stopped.

## 1. Find the code

- Start from something you can see: a CS2 instruction name (`case NAME:` in
  `CS2Interpreter.java`), a text (`LocalizedText`, interface text), a cache index
  (`IndexLoaders.java`), a packet (`PacketDecoder.java`, `Login.java`).
- Follow calls with `grep -rn "static void method8044" .`, then read the whole body.
- Static state lives in unrelated-looking classes. For every static field you depend on, grep
  **all** writers: `grep -rn "Class9.anInt72 = \|anInt72 = " .`. The value may come from a login
  reply, a client parameter (`client.java` switch over the values in `com/Loader.java`) or a
  preference.
- Landmarks:

| What | Where in the reference |
|---|---|
| Client parameters (param 29, 26, …) | `com/Loader.java` (values), `client.java` (which field each sets) |
| Cache indexes and load stages | `IndexLoaders.java` (`loadIndex(IndexReference.INDEX_n…)`, stage blocks) |
| Game states and what each opens | `GameState.setGameState`, `Class348.method6175` (login window) |
| CS2 executor / instructions | `CS2Executor.java`, `CS2Interpreter.executeOperation` |
| Interface fields, decode, render | `IComponentDefinitions.java` (`readValues`, `render`) |
| Layout | `Class484.initSizes`, `Class246.method4204`, `InteractableObject.method16099`, `Class480.method8044` |
| Input and hook queues | `ServerEnvironment.pulseSubInterfaces`, `client.method11768`, `client.java` near line 2790 |
| Login | `Login.java`, `Class455.method7558`, `Class155.method2635` |
| Preferences | `GamePreferences.java` + one class per preference (`getDefaultValue`, `checkValid`) |
| Music | `Node_Sub7` (track), `Node_Sub14` (patch), `MIDIInstrument` (Vorbis), `SoundEffect` (synth), `Node_Sub15_Sub2` (player) |

## 2. Read all of it before writing

Write down every value, the order of operations, and the Java semantics you must keep:

| Java | Rust |
|---|---|
| `int` overflow wraps | `wrapping_add/sub/mul`; `wrapping_shl` (Java masks shift counts to 5 bits) |
| `byte` is signed | `as i8 as i32`; `& 0xff` → `as u8` |
| `a >> b` / `a >>> b` | `a >> b` on `i32` / `((a as u32) >> b) as i32` |
| integer division truncates toward 0 | `/` does the same; floor division is spelled out (`Class371.method6349`) |
| `(char) i` | the low 16 bits as a char |
| `float` vs `double` | keep `f32` vs `f64` exactly as the reference does |
| an exception aborts | return `Err` / stop the script; don't silently continue |
| arrays sized from data | check bounds where Java would throw |

If a part can't be found, or depends on something this client doesn't have (OpenGL/DirectX
toolkits, the applet's web page, Java version checks), port the branch the reference takes when
that thing isn't there, and say which branch in a comment.

## 3. Check what the server must do

The client often only reacts. Login reply codes, interface opens, varps and messages come from
the server; the Rust server (`src/login`, `src/js5`, `src/net.rs`) must send them as the
reference server would. Never fake server data inside the client. If the server side isn't
ported yet, say so in your summary, in `MECHANICS.md` and in `ROADMAP.md`.

## 4. Port into the right module

See the module table in `docs/PORTING.md`. Rules:
- Readable names, with the reference name in the doc comment: ``/// `Class338.method6025`: …``.
- One decoder per cache format in `src/cache/`, each with a unit test on a hand-made sample.
- No dead code: the build denies unused items. Add only what something uses now.
- When you pick between two readings of the reference, or cut a corner, say so in a comment.

## 5. Leave a check, cite, record, commit

- Decoder or formula → a `#[test]` with a tiny input whose answer you worked out from the Java.
- Screen → a `--render-interface` picture (look at it) and, for flows, a self-test step.
- Real cache → a tool run that decodes everything of that kind (like `--export-music`).
- Then `cite-reference`, update `ROADMAP.md`, then `check-and-commit`.
