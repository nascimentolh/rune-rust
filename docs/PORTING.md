# How Rune Rust is ported

The method behind every change in this repository. It needs only the reference Java client,
your own build-727 cache, Rust and Python 3. The skills in `.claude/skills/` walk an assistant
(or you) through the same steps.

## Ground rules

1. **Read the reference first, line by line.** Never guess what the client does or looks like.
   Find the Java that does it, read every line it calls, then port that. If something can't be
   found, say so and leave it out; don't invent a stand-in that looks finished.
2. **Port faithfully, write it readably.** Keep the reference's behaviour (including its quirks:
   integer wrap-around, Java `byte` sign, odd orders), but name things for what they do and put
   the reference name in the doc comment (`/// \`Class338.method6025\`: …`).
3. **Not everything is client-side.** Values that come from the server (login reply codes,
   packets, interface opens) need the server here to send them the reference way. Say plainly
   when the server side is still a stand-in.
4. **Cite.** Every rule lands in `MECHANICS.md` with `file:line` and, for key values, a byte
   offset (`scripts/cite.py`).
5. **Leave one check.** A unit test for a decoder or formula, a self-test step for a screen.
6. **The build is strict.** Unused code fails it; `cargo fmt`, `cargo clippy --all-targets` and
   `cargo test` must pass. Don't silence lints; remove what isn't used yet.

## Reading the reference

- Sources are in `src/main/java/com/jagex/` of the reference client. They aren't valid UTF-8:
  read them as Latin-1 (`iconv -f latin1 -t utf-8 File.java`).
- Names are obfuscated (`Class338`, `method6025`, `anInt7605`). Follow calls with `grep -rn`
  (for example `grep -rn "static void method8044" .`), then read the whole body.
- CS2 instructions: `CS2Interpreter.executeOperation` has `case NAME: helper(exec);`; read the
  helper and every helper it calls (`setComponentPosition`, `popParamsDynamic`, …).
- Game state lives in static fields of unrelated-looking classes (`Class9.anInt72`). Grep for
  every `= ` assignment to a field before deciding its value.

## Finding what is left to port

| Question | Tool |
|---|---|
| What does an interface look like, and which script instructions stop? | `cargo run -- --render-interface 744` (writes `shots/interface-744.png`, prints `script N stopped: OPCODE … not ported`) |
| What happens after clicks and typing? | `RENDER_INPUT="click 382 298; type name; key 84" cargo run -- --render-interface 744` |
| Which scripts call script N or touch component N? | `cargo run -- --scripts-using N` |
| What does a script do? | `cargo run -- --dump-script N` |
| What does an interface contain? | `cargo run -- --dump-interface N` |
| Which scripts/opcodes does a script reach? | `cargo run -- --script-scope N` |
| Music track, patches and samples | `cargo run -- --export-music N` |

Loop: run the tool, take the first `not ported` instruction, read its Java, port it into
`src/client/cs2/ops.rs` (or the module it belongs to), run again.

## Where things go

| Area | Module |
|---|---|
| Cache formats (one decoder per format) | `src/cache/` |
| Script machine and instructions | `src/client/cs2/` |
| Interfaces: components, layout, drawing, input | `src/client/interface/` |
| Game preferences | `src/client/preferences.rs` |
| Server services | `src/js5/`, `src/login/`, `src/net.rs` |
| Developer tools | `src/tools.rs` |

## Citing

```sh
python scripts/cite.py <reference>/src/main/java Class338.java "static int[] anIntArray3973 = {256, 128, 86, 64};"
# jagex/Class338.java:5 @ 0x31  |  static int[] anIntArray3973 = {256, 128, 86, 64};
```

Add the rule to the matching section of `MECHANICS.md` (what, reference function, our module)
and, for values, a row in the offset table: `| Floor-1 ranges 256, 128, 86, 64 | \`jagex/Class338.java\` @ 0x31 | how it was found |`.

## Checking and committing

```sh
cargo fmt && cargo clippy --all-targets && cargo test
cargo run -- --self-test shots/selftest.json   # needs server.cfg pointing at your cache
```

Commits: one topic each, imperative subject, body says what was ported and from which reference
functions. No `Co-Authored-By` lines. Never commit cache files, keys, `server.cfg` or `shots/`.
