---
name: check-and-commit
description: Verify a change the way the project requires (fmt, clippy, tests, self-test, a look at the screen) and commit it without co-author lines. Use at the end of every change.
---

# Check and commit

1. `cargo fmt`
2. `cargo clippy --all-targets` — must be clean. Unused code fails the build: remove it rather
   than allowing the lint.
3. `cargo test`
4. With a cache configured: `cargo run -- --self-test shots/selftest.json` and, for screens,
   `cargo run -- --render-interface <id>`; open the PNG and compare with what the reference shows.
   If a running "Game server" window holds `target/debug/rune-rust.exe`, build with
   `CARGO_TARGET_DIR=target/alt` or close that window.
5. Update `MECHANICS.md` (what's done, what isn't) and the status table in `.github/README.md`
   when a feature changes state.
6. Commit one topic: imperative subject; the body names what was ported and from which reference
   functions. **No `Co-Authored-By` lines.** Never commit cache files, keys, `server.cfg`,
   `data/` or `shots/`.
