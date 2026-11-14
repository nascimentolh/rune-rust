---
name: extend-server
description: Add to the server the way docs/SERVER.md requires — a client packet handler, a server packet, a tick stage, player state or a lobby answer — with typed opcodes documented on their table row, no locks in the world, a test, and docs/PROTOCOL.md regenerated. Use for any change under src/server/ or to a packet in src/protocol/.
---

# Extend the server

The design and its rules are in `docs/SERVER.md`. This is the checklist, with the traps.

## 0. Before you start

- Read `docs/SERVER.md` once per session. Check `docs/PROTOCOL.md` for the packet: it lists
  every id, its size, and whether it is ported.
- There is no reference server. The *bytes* come from the reference client (how it reads a
  server packet, how it writes a client packet); *what to send and when* is this server's
  choice, made from what the client and the cache's scripts wait for. Say which in a comment.
- Never guess a layout. If the reference's reader can't be found, stop and say so.

## 1. Where it goes

| Adding | Goes in |
|---|---|
| A packet's id, name, size | already there: the row in `src/protocol/server_prot.rs` / `client_prot.rs` |
| A server packet's payload | encoder in `src/protocol/server_packets.rs` (large layouts: their own file) |
| A client packet's payload | encoder **and** decoder side by side in `src/protocol/client_packets.rs` |
| What the world does with a client packet | a function in `src/server/world/handlers/<area>.rs` + a row in `handlers::handle` |
| What a player is sent on entering | `src/server/world/enter.rs` |
| Which interface opens where | `src/server/world/interfaces.rs` |
| Per-player game state | a field of `Player` (`src/server/world/player.rs`) |
| Something every tick, or touching several players | a stage: its own file under `src/server/world/`, called from `Game::tick` between read and flush |
| A lobby answer | a row in `Lobby::answer` (`src/server/lobby/mod.rs`) |
| A login rule or reply code | `src/server/login/` (codes in `src/protocol/login.rs`) |

## 2. Write it

- **Packet first.** Encoder/decoder with a test of the exact bytes, worked out from the
  reference's reader (`readIntV2` = bytes 2, 1, 4, 3; `readUnsignedShortLE128` = low byte
  + 128 then high; `readUnsignedByteC` = negated; `read128Byte` = 128 − value).
- **Document the row.** Put a `///` comment on the packet's row in the table: the payload as
  the reference reads or writes it (with its function names), the encoder/decoder here, who
  sends or handles it. That comment *is* the packet's documentation and marks it as ported.
- **Then the handler or stage.** It takes `&mut Player` (a stage: what it needs of `Game`),
  reads the payload through the decoder, changes state, queues with `player.send(..)` /
  `player.send_all(..)`. To end a session: `player.log_out(..)` or `player.leaving = true`.
- **Test it without a socket**: `Player::new(1, "bob".into(), None)`, call the function,
  compare `player.take_queued()`.

## 3. Traps

| Trap | Instead |
|---|---|
| A packet id as a number (`85`, `0x55`) | `ServerProt::Region`; ids exist only in the tables |
| Building bytes in a handler (`to_be_bytes`, `vec![..]` payloads) | an encoder in `src/protocol/` (a test fails otherwise) |
| `Mutex`/`Arc<..>` state so a handler can reach it | the state belongs to `Player` or `Game`; only the world's thread touches it |
| Writing to the stream from the world, sleeping, reading a file in a handler | queue packets; slow work doesn't belong in the tick |
| A handler changing another player | record the request on this player; a tick stage applies it for everyone in index order |
| `payload[3]`, `.unwrap()` on client data | decoders return `Option`; ignore what doesn't parse |
| Adding the encoder "for later" | unused code fails the build; add it with its first user |
| Editing `docs/PROTOCOL.md` | it is generated: `cargo run -- --protocol-doc` |
| Answering in the lobby style (at once) from the world | world answers go out at the end of the tick; only the lobby answers at once |
| `use crate::client::..` in the server (or the reverse) | share it through `src/protocol/` (a test fails otherwise) |

## 4. Finish

```sh
cargo run -- --protocol-doc      # if a table row or its comment changed
cargo fmt && cargo clippy --all-targets && cargo test
cargo run -- --self-test shots/selftest.json
```

Then `cite-reference` for the values taken from the reference, update `ROADMAP.md` and
`MECHANICS.md`, and `check-and-commit`. If the change needed a new rule or bent one, update
`docs/SERVER.md` in the same commit.
