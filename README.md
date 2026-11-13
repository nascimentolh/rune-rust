This build contains the original game's own files, for the private use of the person who owns the game, on their own computer. It must not be shared or published.

# Rune Rust: a Rust client + server for the open727 client

A port of the build-727 reference client to readable Rust, with a small server to run it
against. One program, run as two processes:

- **Client** (default): connects to the server and plays the reference client's flow.
- **Server** (`--server`, add `--console` for a terminal): serves the cache (JS5), logins, the
  lobby and the first world packet on port 5555.

## Status

You can log in, go through the lobby and enter the world, where the map around the spawn
(Lumbridge) is drawn like the original. You can't walk or use the game interface yet.

| Part | Status | Notes |
|---|---|---|
| Cache download (JS5) and loading screens | Done | CRC-checked, the reference's loading screens |
| Login screen (interface 744) | Done | Its own scripts, login music, Auto Setup, options |
| Login to the server (RSA, ISAAC) | Done | Accounts with passwords on the server |
| Lobby | Done | Window and panels, world list, PLAY, log out; no world pings |
| Window | Done | Resizable, maximisable, screen size modes |
| World: ground | Done | Four floors, floor shapes, ground blending, lighting |
| World: camera | Partly | Pitch, lift and Ctrl + wheel zoom; no turning yet |
| World: map objects | Done | Walls, decorations, scenery, roofs, ground decoration, lit models |
| World: roofs over the player | Done | Hidden under roofs and behind them; bridges |
| Textures (water, leaves, roof tiles) | Partly | Every texture can be generated like the original (all the operations the textures use). Not on screen yet: those parts still show plain colours |
| Game packets, walking, game interface | Not yet | Next after textures |
| Chat, friends, NPCs, other players | Not yet | |
| Sky, fog, shadows, animations | Not yet | |

## What works

- **Loading**: the cache is downloaded from the server and CRC-checked, with the reference
  client's loading screens.
- **Login screen**: interface 744 from the cache, drawn by its own scripts, with the login music
  and its jingle. Auto Setup picks the graphics level and toolkit as the original does.
- **Lobby**: the lobby window and its panels from the cache; PLAY enters the world, the close
  button logs out. The custom top bar is off (`lobby_top_bar` in `server.cfg`).
- **Window**: resizable and maximisable, with the reference's screen size modes.
- **World**:
  - the ground of the 104 × 104 tiles around the player, all four floors, read from the map
    (`m{x}_{y}`);
  - floor colours, floor shapes and ground blending between floors;
  - lighting as the OpenGL toolkit does it;
  - the reference's camera (Ctrl + mouse wheel zooms).
- **Map objects**: every object around the player is read (`l{x}_{y}`, decrypted with the
  region keys the server sends) with its definition. On every floor these are drawn with their
  models, placed, fitted to the ground and lit like the original: walls, doors and windows, wall
  decorations (banners, torches, paintings), scenery (trees, bushes, fountains, barrels...),
  roofs and ground decoration.
- **Roofs**: under a roof, or with a roof between the camera and you, the floors above are
  hidden as the original does (the remove-roofs preference), bridges included.

See `ROADMAP.md` for what comes next, and `MECHANICS.md` for every ported rule with the
reference file and offset it comes from.

## Running

1. Put the build-727 cache folder where `server.cfg` says (`cache = ...`), and the map keys in
   `data/keys.json`. Copy `server.cfg.example` to `server.cfg` the first time.
2. Start the server: `cargo run --release -- --server --console`.
3. Start the client: `cargo run --release`.

Other options:
- `--config <file>` uses another settings file.
- `--self-test <result.json>` plays the whole flow against a server of its own: loading, login,
  lobby, logout, PLAY into the world (with a check of the roof hiding). It saves pictures beside
  the result, the world in `world.png`.
- `--gen-rsa` makes a new RSA key pair for client and server.
- Checks of the cache data, each taking a number:
  - `--map-terrain <region>` and `--map-locations <region>`, where region is (x << 8) | y;
  - `--texture-stats 0`: the texture table and every texture's operations;
  - `--render-texture <id>`: a texture as `shots/texture-<id>.png`, how many can be drawn and
    the first that can't;
  - `--render-music <id>` and `--export-music <id>`;
  - `--dump-interface`, `--render-interface`, `--dump-script`, `--scripts-using`, `--script-scope`.

## Files

Settings are in `server.cfg`. The cache is read at run time from the folder set there; nothing
from it is copied into this folder except `data/keys.json` and the self-test's pictures in
`shots/`. Never commit game files, keys, `server.cfg`, `data/`, `shots/` or `preferences.dat`.
See `CREDITS.md` for every original file used.
