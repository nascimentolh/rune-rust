# Rune Rust

A readable, well-structured Rust port of the **build-727 game client** (the open727Client
deobfuscated Java client), plus a small Rust **server** that serves your own cache to it.

- **Faithful first:** every behaviour is read line by line from the reference Java source and
  cited (`file:line`) in [`MECHANICS.md`](../MECHANICS.md). Nothing is invented.
- **Clean code:** small single-purpose modules; unused or dead code fails the build; `clippy`
  and `rustfmt` are enforced.
- **No game files in this repository.** You bring your own cache; the client draws the loading
  and login screens from it at run time.

## Status

| Part | State |
|---|---|
| Cache reading (data file, indexes, containers, reference tables, XTEA keys) | done |
| JS5 cache server and client (727 handshake, requests, 512-byte blocks) | done |
| Loading screens (fallback bar, index-33 screens, fonts, crossfades, stage texts) | done |
| Login protocol (727 world login, RSA + XTEA, ISAAC) | done |
| Login screen: interface and script formats, opcode table | done |
| Login screen: CS2 script machine, interface layout and 2D rendering (window 744 drawn by its own scripts) | done |
| Login screen: mouse and keyboard on the window (log in through the real form) | done |
| Login screen: connecting spinner and login error messages, drawn by the window's scripts | done |
| Login screen: Auto Setup, Options panel (detail presets, volume sliders, stereo/mono), sound button | done |
| Login screen: login music (MIDI synthesizer), 3D scene behind it | not started |
| Game world after login | not started |

## Recent changes

- **Login window from the cache.** The client opens the login window (interface 744, named in the
  cache's entity defaults) and runs its own CS2 scripts: a script machine (`client/cs2/`), live
  interfaces with layout and 2D drawing (`client/interface/`), enums, params, structs and the
  game preferences of a fresh install. Every start-up script of the window runs to its end.
- **New tool:** `--render-interface <id>` draws any interface from your local cache into
  `shots/interface-<id>.png`, for checking a port without a server.
- **Self-test:** a `login-window` step checks the window arrives over JS5 and is drawn.
- **Mouse and keyboard on the login window.** Hover, click and typing reach the window's own
  scripts once per 20 ms tick, as in the reference; you log in through the real form.
- **Login messages.** The window shows "Logging In - Please Wait" and the server's refusal
  ("Invalid login or password." …) through its own scripts, from the reply code the client keeps.
- **Auto Setup and Options.** Auto Setup measures the CPU and applies the reference's detail
  preset; the Options panel's buttons, volume sliders and stereo/mono work.
- **Next:** the login music (the client's MIDI synthesizer), then the 3D scene behind the window.

## Build and run

Needs Rust (stable). On Linux also: `libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev`.

```sh
cp server.cfg.example server.cfg      # then set `cache` to your cache folder
cargo build --release
cargo run --release                   # the client; it starts the server if none is running
cargo run --release -- --server       # just the server (window); add --console for a terminal
```

Useful flags: `--gen-rsa` (new login key pair), `--self-test <file.json>` (scripted run of the
whole flow), `--screenshot <file.png>`, and for porting: `--dump-interface <id>`,
`--dump-script <id>`, `--script-scope <id>`, `--scripts-using <n>`, `--export-music <id>`, `--render-interface <id>` (draws an interface from
the local cache into `shots/`).

The cache must be **build 727** (each build renumbers its script opcodes), for example
[OpenRS2 cache #309](https://archive.openrs2.org/caches/runescape/309/disk.zip), with its map keys
([keys.json](https://archive.openrs2.org/caches/runescape/309/keys.json)) in `data/`.

## Layout

| Module | Job |
|---|---|
| `cache/` | file formats: store, containers, reference tables, archives, sprites, fonts, loading screens, interfaces, scripts, enums, params, XTEA, keys |
| `client/` | updater, loading stages and screens, software raster and fonts, interfaces, the CS2 script machine, preferences, login |
| `js5/`, `login/`, `net.rs` | the server's cache and login services |
| `server_app.rs`, `loader.rs`, `screen.rs` | running the server process |
| `selftest.rs` | end-to-end check against a real or sample cache |

## Contributing

See [`CONTRIBUTING.md`](../CONTRIBUTING.md), the roadmap in [`ROADMAP.md`](../ROADMAP.md) and the porting method in
[`docs/PORTING.md`](../docs/PORTING.md) (also as AI-assistant skills in
[`.claude/skills/`](../.claude/skills/)). In short: read the reference first, cite it, keep
modules small, and `cargo fmt && cargo clippy --all-targets && cargo test` must pass.

## Legal

This is an independent, non-commercial interoperability project. It contains no game assets,
caches or keys. RuneScape is a trademark of Jagex Ltd; this project is not affiliated with or
endorsed by Jagex. Use it only with files you are entitled to use.

Licensed under the [MIT License](../LICENSE).
