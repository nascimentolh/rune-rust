---
name: check-and-commit
description: Verify a change the way the project requires (fmt, clippy, tests, tool runs, self-test, looking at the screen), update the docs and ROADMAP, and commit without co-author lines. Use at the end of every change.
---

# Check and commit

## Checks (all must pass)

```sh
cargo fmt
cargo clippy --all-targets      # clean: Cargo.toml lints deny unused code and every clippy warning
cargo test
cargo run -- --self-test shots/selftest.json   # with server.cfg pointing at your build-727 cache
```

- Unused code is a build error. Don't add `#[allow(...)]`; remove the item or use it.
- Also run the tool for what you changed (`--render-interface`, `--export-music`, …) and **look
  at the output**: open the PNG; for audio, listen to the WAV or at least check peak and RMS.
- For a cache format, decode *every* entry of that kind in the real cache, not just one.

## Common problems

| Symptom | Cause / fix |
|---|---|
| `failed to remove file target\debug\rune-rust.exe … Access denied` | a "Game server" window started by the client holds the exe: close it, or build with `CARGO_TARGET_DIR=target/alt` |
| Scripts decode as nonsense, the login screen is wrong | the cache isn't build 727 |
| Map archives don't decrypt | the keys file is missing or belongs to another cache (`keys` in `server.cfg`) |
| Self-test can't log in | server unreachable or RSA keys don't match: see `server.cfg.example` |
| Garbled text reading Java | read it as Latin-1 (`iconv -f latin1 -t utf-8`) |
| A shell overwrite is refused by your tool's safety mode | write the file with your editor/file tool instead of `cat >` |

## Docs

- `MECHANICS.md`: what was ported, from which functions, what is still missing.
- `ROADMAP.md`: move the item to done, add what you found is left and where it lives.
- `.github/README.md`: status table and "Recent changes" when a feature changes state.

## Commit

- One topic per commit; imperative subject (≤ 72 characters); the body says what was ported,
  from which reference functions, and what's still missing.
- **Never add `Co-Authored-By` lines.**
- Never commit cache files, keys, `server.cfg`, `data/`, `shots/` or `target/` (all ignored).
- Don't push or open pull requests unless the maintainer asks.
