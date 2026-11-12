# Rune Rust

A readable, well-structured Rust port of the **build-727 game client** (the open727Client
deobfuscated Java client), plus a small Rust **server** that serves your own cache to it.

- **Faithful first:** every behaviour is read line by line from the reference Java source and
  cited (`file:line`) in [`MECHANICS.md`](../MECHANICS.md). Nothing is invented.
- **Clean code:** small single-purpose modules; unused or dead code fails the build; `clippy`
  and `rustfmt` are enforced.
- **No game files in this repository.** You bring your own cache; the client draws the loading
  screens from it at run time.

## Status

| Part | State |
|---|---|
| Cache reading (data file, indexes, containers, reference tables, XTEA keys) | done |
| JS5 cache server and client (727 handshake, requests, 512-byte blocks) | done |
| Loading screens (fallback bar, index-33 screens, fonts, crossfades, stage texts) | done |
| Login protocol (727 world login, RSA + XTEA, ISAAC) | done |
| Login screen (interface 744 + CS2 scripts) | in progress |
| Game world after login | not started |

## Build and run

Needs Rust (stable). On Linux also: `libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev`.

```sh
cp server.cfg.example server.cfg      # then set `cache` to your cache folder
cargo build --release
cargo run --release                   # the client; it starts the server if none is running
cargo run --release -- --server       # just the server (window); add --console for a terminal
```

Useful flags: `--gen-rsa` (new login key pair), `--self-test <file.json>` (scripted run of the
whole flow), `--screenshot <file.png>`.

The cache must be build 718–727 in the `main_file_cache.dat2` / `.idx` layout (for example from
[archive.openrs2.org](https://archive.openrs2.org)); map keys go in `data/keys.json`.

## Layout

| Module | Job |
|---|---|
| `cache/` | file formats: store, containers, reference tables, archives, sprites, fonts, loading screens, XTEA, keys |
| `client/` | updater, loading stages and screens, software raster and fonts, login |
| `js5/`, `login/`, `net.rs` | the server's cache and login services |
| `server_app.rs`, `loader.rs`, `screen.rs` | running the server process |
| `selftest.rs` | end-to-end check against a real or sample cache |

## Contributing

See [`CONTRIBUTING.md`](../CONTRIBUTING.md). In short: read the reference first, cite it, keep
modules small, and `cargo fmt && cargo clippy --all-targets && cargo test` must pass.

## Legal

This is an independent, non-commercial interoperability project. It contains no game assets,
caches or keys. RuneScape is a trademark of Jagex Ltd; this project is not affiliated with or
endorsed by Jagex. Use it only with files you are entitled to use.

Licensed under the [MIT License](../LICENSE).
