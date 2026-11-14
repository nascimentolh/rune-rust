# The server: how it is built, and how to add to it

Everything under `src/server/` follows this document. Read it before changing the server; if
a change doesn't fit, change the document in the same commit and say why. The rules that can be
checked by a machine are checked by `cargo test` (`src/architecture.rs`,
`protocol::doc::tests`), so a change that breaks them doesn't build green.

There is no reference server. The client is ported from the reference; what the server sends
is the server's own choice, made from what the client and the cache's scripts wait for. Every
such choice says so in a comment ("this server's choice", "estimated"). Never present a guess
as something the reference does.

## The shape

```
        a port per world (the first world's is the server's own, 5555)
                       │  first byte picks the service
        ┌──────────────┴──────────────┐
     JS5 (cache)                    login
     src/server/js5.rs              src/server/login/
                                    checks, account, reply
                                      │
                         ┌────────────┴────────────┐
                      lobby                      world N
                      src/server/lobby/          src/server/world/
                      answered at once,          the world whose port the client
                      per connection             came in on: one thread owns every
                                                 player in it; 600 ms tick
```

A server runs as many worlds as its administrator opens. Each is a `World` with its own port,
thread, seats and players; two worlds share nothing but the accounts (a name can be in one
world at a time) and the map keys. The first world is the one `server.cfg` describes
(`port`, `world_capacity`); the others are opened from the status window ("Open a world": a
name, a port or the next free one, how many players, the place its flag and location name
stand for, and the Members, Loot share, Themed, Hidden and Quick chat settings), saved to the
`worlds` file as JSON and opened again at start-up.

Every option is something the client is told: a bit of the world list's flags, the activity or
location text, or a field of the login reply (`Options::flags`, `Options::activity`,
`Options::world_info`). Nothing is kept that the client never sees, so a new option means
finding where the client reads it first — `MECHANICS.md`, "What a world says about itself".

| Path | What lives there |
|---|---|
| `src/protocol/` | The wire format, shared with the client: every packet id, size and payload. Nothing else in the project builds or reads packet bytes. |
| `src/server/app.rs`, `loader.rs`, `screen.rs`, `trace.rs` | Starting the process, start-up steps, the status window, the byte-level log. |
| `src/server/net.rs` | The ports: a listener per world, one thread per connection, picks the service. `Stats` and `Ctx` (counters and the log every part reports to). |
| `src/server/worlds.rs` | The worlds that are running, what each was opened with (`Options`), and the file they are saved to. |
| `src/server/panel.rs` | The administrator's part of the status window: the list of worlds and the form that opens one. |
| `src/server/flags.rs` | The country flags of the world list, read from the cache so the panel can offer them as pictures. |
| `src/server/session.rs` | Reading a logged-in client's packets (id, size, payload). Used by the lobby and the world. |
| `src/server/js5.rs` | The cache service. |
| `src/server/login/` | One login: read, check (build, account, a free seat in the world whose port it came in on), reply, then hand the connection to the lobby or that world. Decides nothing after the reply. |
| `src/server/lobby/` | A lobby player. No shared state, so no tick: each packet is answered on the connection's own thread. |
| `src/server/world/` | A game world (below). |

The client (`src/client/`) and the server never use each other. They meet only in
`src/protocol/`.

## The world

**Only the world's thread reads or changes game state.** That is the whole concurrency design:
no game state is behind a lock, so there are no lock orders to get wrong and no two players
ever see half an update.

```
client ──TCP──▶ reader thread ──inbox (64 packets)──▶ ┐
                                                      │  world thread
client ◀─TCP── writer thread ◀─outbox (8 ticks)────── ┘  (owns every Player)
```

- **Reader** (the connection's own thread, `world/connection.rs`): decodes client packets into
  the player's inbox. A full inbox makes it wait, so a flooding client is slowed by TCP and
  can't grow the server's memory or a tick's work.
- **World thread** (`world/mod.rs`, `Game`): every 600 ms (`TICK`) runs the tick. Between
  ticks it only takes in players the login hands over, so logging in doesn't wait for a tick.
- **Writer** (`world/connection.rs`): gets one buffer per player per tick and writes it. The
  world never writes to a socket, so a client that stops reading can't hold up the tick; its
  outbox fills and the world lets the player go.

The tick, in order (`Game::tick`):

1. **Read**: for each player in index order, the packets that arrived, each through its
   handler (`world/handlers/`).
2. *(Stages that move the world on go here: movement, then what each player sees of the
   others. Each is a loop over the players in index order.)*
3. **Flush**: everything queued for a player is framed and handed to their writer as one
   buffer.
4. **Leave**: players who logged out, hung up or fell too far behind are let go. Dropping the
   player frees their index and their name, and closes their connection after what was queued
   (a logout packet, say) is written.

Files:

| File | What |
|---|---|
| `world/mod.rs` | The `World` handle, the thread, entering, the tick, leaving. |
| `world/seats.rs` | Player indices 1..=2047. The one lock in the world: login threads take a seat (the index goes in the login reply) and the world keeps it with the player. |
| `world/connection.rs` | The two threads and two queues of one player. The only file in `world/` that touches a socket. |
| `world/player.rs` | `Player`: plain game state and the packets queued for them. No socket, no lock: make one in a test and look at what it queued. |
| `world/enter.rs` | What a player is sent on entering, in the order the client needs. |
| `world/interfaces.rs` | Which interface is opened where in the game's window. |
| `world/handlers/` | One function per client packet the world understands, a file per area of the game. |

### How many players

A world holds up to `world_capacity` players (`server.cfg`; at most 2047, the client's player
table). The cost per player is two threads (the writer's stack is 128 KB) and two bounded
queues; the tick's cost is the sum of the handlers and stages, all on one thread. A test logs
500 players in over real connections at once and has each one answered and logged out by the
tick (`server::login::tests::a_world_of_500_players_is_served_by_the_tick`).

The status window shows how long the last tick took and the slowest so far; the log gets a line
whenever a tick takes longer than 600 ms. Watch those when adding a stage: the budget for all
stages together is one tick.

Not done yet, and where it goes when needed:
- The client entering a world other than the first: the world list has no port in it (a world
  is a number and a host), and this client logs in to its own server's address whatever world
  was picked. How the reference turns the picked world into an address must be ported in the
  client (`Class62.setGameHost` and what uses it) before another world's port is reached.
- Closing a world from the status window (today: delete its line from the `worlds` file while
  the server is stopped).
- Thousands of connections: only `world/connection.rs` and `net.rs` know about threads and
  sockets, so they can change to another I/O model without touching a handler.

## Rules

1. **Packets live in `src/protocol/`.** Ids and sizes are rows of `ServerProt` / `ClientProt`;
   a payload is built by an encoder in `server_packets.rs` and read by a decoder in
   `client_packets.rs`. The server never writes a packet id as a number and never assembles
   or picks apart payload bytes.
2. **A ported packet is documented on its row.** The doc comment on the table row says what
   the payload is (as the reference reads or writes it, with the reference's function names),
   which function here encodes or decodes it, and who sends or handles it. `docs/PROTOCOL.md`
   is written from those rows: run `cargo run -- --protocol-doc` after changing one.
3. **Game state belongs to the world's thread.** No `Mutex`, `RwLock` or atomic in
   `src/server/world/` outside `seats.rs`. Other threads talk to the world through its queues.
4. **Handlers are small and never wait.** A handler gets the player and the payload, changes
   the player, queues packets with `player.send(..)`. No socket, no sleep, no file. Anything
   that touches other players is a tick stage, not a handler: the handler records what the
   player asked for, and the stage carries it out for everyone in index order.
5. **Trust nothing from the client.** A short or senseless payload is ignored (decoders return
   `Option`); never index into a payload, never `unwrap` on client data.
6. **No dead code.** The build denies unused items. Add a packet's encoder, handler or state
   when something uses it, not before. A packet with no handler is simply not in
   `handlers::handle`.
7. **Leave a check.** A handler or stage gets a unit test that makes a `Player`, runs it and
   compares `take_queued()`. An encoder gets a test of its bytes against the reference's
   reader.
8. **Say what is the server's choice.** Which interface opens where, starting values, messages:
   a comment says how the choice was made (from which script or component of the cache).

## Recipes

### A client packet the world should understand

1. Find how the client writes it in the reference (`ClientProt.java` for the id and size, then
   the code that sends it). Read every line.
2. `src/protocol/client_packets.rs`: add the encoder the client uses and, beside it, a decoder
   for what the server needs from the payload (only the fields something reads). Test both
   together.
3. `src/protocol/client_prot.rs`: put a doc comment on the packet's row (payload, encoder and
   decoder, who handles it).
4. `src/server/world/handlers/`: add a function in the file for its area (a new file for a new
   area), and a row for it in `handlers::handle`. Test it with a `Player`.
5. `cargo run -- --protocol-doc`, then cite the reference (`cite-reference`) and check
   (`check-and-commit`).

### A server packet

1. Find how the client reads it (`PacketDecoder.java`, `ServerProt.java`). Mind the reference's
   byte orders (`readIntV2`, `readUnsignedShortLE128`, `readUnsignedByteC`…).
2. `src/protocol/server_packets.rs`: add an encoder returning `Packet`, with a test of its
   bytes. A packet with a large layout gets its own file (`region.rs`, `world_list.rs`).
3. `src/protocol/server_prot.rs`: doc comment on the row (payload, encoder, who sends it).
4. Send it from where it belongs: `world/enter.rs` (on entering), a handler, or a tick stage.
   The client side (`src/client/lobby.rs`, `apply`) must read it the reference's way.
5. `cargo run -- --protocol-doc`, cite, check.

### Something every tick (movement, updates, timers)

1. Add the state to `Player` (`world/player.rs`).
2. Add the stage as a function in its own file under `world/` taking what it needs of `Game`,
   and call it from `Game::tick` between reading and flushing. Players in index order.
3. Test it without sockets: build the players, run the stage, look at what was queued.
4. Check the tick time on the status window with many players.

### An option for worlds

1. Find where the client reads it: a flag bit or field of the world list (the World Select's
   scripts, 3115 and 3117), or a byte of the login reply. Write down what it does there.
2. Add the field to `Options` (`src/server/worlds.rs`) with a doc comment saying that, plus
   its `check` rule if it can be wrong; it saves and loads itself (serde).
3. Put it where the client reads it: `Options::flags` / `Options::activity` (world list),
   `Options::world_info` (login reply), or `World::start` if the world itself behaves
   differently.
4. Add its control to `src/server/panel.rs` (`FIELDS` or `BOXES` and `Panel::options`), and a
   letter in the worlds table if it is worth seeing at a glance.
5. Only offer what something reads: an option with no effect is dead code.

### A lobby packet

Add a row to `Lobby::answer` (`src/server/lobby/mod.rs`); the encoder and decoder go in
`src/protocol/` as above.

## Checking

```sh
cargo fmt && cargo clippy --all-targets && cargo test
cargo run -- --protocol-doc                     # after touching a packet table
cargo run -- --self-test shots/selftest.json    # client and server together, real cache
```
