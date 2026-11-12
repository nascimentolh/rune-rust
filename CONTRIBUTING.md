# Contributing to Rune Rust

Thanks for helping. The goal is a **faithful** port of the build-727 client: readable Rust that
does exactly what the reference Java client does.

## How to port (no special tools needed)

[`docs/PORTING.md`](docs/PORTING.md) is the full method: reading the obfuscated reference,
finding what's left with the project's own tools (`--render-interface`, `RENDER_INPUT`,
`--scripts-using`, `--dump-script`, `--script-scope`, `--export-music`), citing with
`scripts/cite.py`, and checking. The same steps are packaged as skills for AI assistants in
[`.claude/skills/`](.claude/skills/): `port-from-reference`, `port-cs2-instruction`,
`cite-reference` and `check-and-commit`. Claude Code picks them up from this folder; with
other assistants, point them at the `SKILL.md` files.

## The one rule: read the reference first

1. Find the code in the reference (`open727Client`, `src/main/java/com/...`) that does what you
   are porting. Read it line by line, including the helpers it calls.
2. Port it; don't redesign it. Keep the reference's numbers, order and quirks. When it has a bug
   that changes what you see, keep the bug and say so in a comment.
3. Cite it: a doc comment naming the class and method (`Class299.method187`), and a row in
   `MECHANICS.md` (and `mechanics.json` for numbers) with `file:line`.
4. If you can't find it in the reference, don't guess. Open an issue and leave a clear
   "not ported yet" note.

## Code standard

- Small modules with one job each; a module doc comment says what it is and what it ports.
- `Cargo.toml` denies unused and dead code, forbids `unsafe`, and denies `clippy::all`. Don't
  silence a lint with `allow` to get past it: use the code or remove it. The only accepted
  `allow` is `clippy::too_many_arguments` on a function that mirrors a reference signature, with
  a comment.
- Plain names in the Rust code (`draw_paragraph`, not `method373`); the reference names go in the
  doc comments.
- Every format decoder gets a small test with real bytes (from a cache dump or a hand-built sample).

Before you push:

```sh
cargo fmt
cargo clippy --all-targets
cargo test
cargo run -- --self-test target/selftest.json   # needs a cache in server.cfg, or uses a sample
```

## Commits and pull requests

- One topic per commit, written in the imperative ("Port the login interface decoder").
- The body says what reference code was ported (`Login.java:165-266`).
- No game files in commits: no caches, sprites, screenshots of game art, or keys.
- Open a pull request against `main`; the template asks what you ported and how you checked it.

## Reporting problems

Open an issue with what you ran, what you expected (with the reference `file:line` if you know
it), and what happened. For connection problems, attach the relevant lines of `logs/trace.log`
(the server's byte trace), with passwords removed.

By contributing you agree your work is released under the [MIT License](LICENSE), and you follow
the [Code of Conduct](CODE_OF_CONDUCT.md).
