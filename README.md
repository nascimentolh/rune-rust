This build contains the original game's own files, for the private use of the person who owns the game, on their own computer. It must not be shared or published.

# Rust client + server for the open727 client

One program, run as two processes:

- **Client** (default): the reference client's loading screens, drawn from the cache it downloads
  from the server; then a login screen and login.
- **Server** (`--server`, `--console` for a terminal): serves the cache (JS5) and logins on port 5555.

Settings are in `server.cfg`. The cache is read at run time from the folder set there; nothing from
it is copied into this folder except `data/keys.json` and the self-test's pictures in `shots/`.
See `CREDITS.md` for every original file used, and `MECHANICS.md` for what is ported and what isn't.
