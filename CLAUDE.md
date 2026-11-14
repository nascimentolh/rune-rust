# Rune Rust

A Rust port of the build-727 reference client, with a server to run it against. One program,
two processes (`cargo run` is the client, `cargo run -- --server --console` the server).

## Read before you change anything

| Changing | Read first | Skill |
|---|---|---|
| The client, a cache format, a screen | `docs/PORTING.md` | `port-from-reference`, `port-cs2-instruction`, `port-cache-format` |
| The server (`src/server/`) | `docs/SERVER.md` | `extend-server` |
| A packet (`src/protocol/`) | `docs/SERVER.md` (rules 1–2, recipes), `docs/PROTOCOL.md` | `extend-server` |
| Anything, at the end | — | `cite-reference`, `check-and-commit` |

`ROADMAP.md` says what is done and what is next; `MECHANICS.md` holds every ported rule with
its reference file and offset.

## Rules that are never broken

- **Port, don't invent.** Read the reference Java line by line before writing. What the server
  decides for itself (there is no reference server) is marked as its own choice in a comment.
- **Three sides.** `src/client/` and `src/server/` never use each other; they meet only in
  `src/protocol/`, which holds every packet id, size and payload.
- **Packets are typed.** `ServerProt::Region`, `ClientProt::ScreenSize`: never a packet id as a
  number, never payload bytes assembled outside `src/protocol/`. A ported packet has a doc
  comment on its table row; `docs/PROTOCOL.md` is generated from the rows
  (`cargo run -- --protocol-doc`), never edited by hand.
- **The world's thread owns game state.** No locks in `src/server/world/` (except `seats.rs`).
  Handlers change the player and queue packets; they never touch a socket or wait.
- **No dead code.** The build denies unused items. Don't add `#[allow(...)]`; add things when
  something uses them.
- **Leave a check** for whatever you add (a unit test, a self-test step).

`cargo test` enforces the three-sides, world-thread and protocol-document rules
(`src/architecture.rs`, `src/protocol/doc.rs`). If one fails, move the code; don't edit the test.

## Checks

```sh
cargo fmt && cargo clippy --all-targets && cargo test
cargo run -- --self-test shots/selftest.json   # needs server.cfg pointing at a build-727 cache
```

## Commits

One topic each, imperative subject, body says what changed and from which reference functions.
**No `Co-Authored-By` lines.** Never commit cache files, keys, `server.cfg`, `data/`, `shots/`
or `preferences.dat`. Don't push unless asked.
