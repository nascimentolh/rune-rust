---
name: port-cache-format
description: Add a decoder for a cache file format (configs, sprites, music, sounds, interfaces…) to src/cache, faithful to the reference's decode method, tested on a hand-made sample and on every entry of the real cache. Use when a feature needs data the client doesn't read yet.
---

# Port a cache format

1. **Find the loader** in `IndexLoaders.java`: which index (`IndexReference.INDEX_n`) and how
   files are addressed:
   - `getFile(archive, file)`; shared configs split an id as `archive = id >>> bits`,
     `file = id & ((1 << bits) - 1)` (`SharedConfigsType`: enums use 8 bits; params are archive 11
     and structs archive 26 of index 2, one file per id);
   - `getFile(id)` means file `id` of archive 0 when the index has one archive, otherwise
     archive `id`, file 0 (`Index.getFile(int)`, same rule in `Class250.method4287/4290`).
2. **Read the decode method** (`decode`, `readValues`, the constructor taking a `ByteBuf`) and
   every `ByteBuf` helper it uses. Check each one; obfuscated names lie:
   - `readUnsignedByte/Short`, `readInt`: big-endian; `read24BitUnsignedInteger`;
   - `readBigSmart` (2 or 4 bytes), `method13099` = 7-bit variable-length number,
     `readString` = Latin-1 up to a 0 byte;
   - opcode loops: check what happens on an unknown opcode (some skip it without reading!);
   - bit readers (Vorbis) are least-significant-bit first.
3. **Write `src/cache/<format>.rs`** with `buffer::Reader`; return `Result` with `corrupt(...)`
   where the reference would throw. Keep Java integer and byte semantics (see
   `port-from-reference`). Archives come by **file id** (`client::source::Files`, gaps are `None`),
   not by position.
4. **Test** it with a hand-built byte array whose result you computed from the Java.
5. **Decode everything** of that kind from the real cache in a tool (add it to `src/tools.rs` and
   the table in `src/main.rs`), print counts, and fix until all decode. Example:
   `cargo run -- --export-music 0` decodes all 247 patches and 1,187 Vorbis samples.
6. Wire it where the client needs it (usually `client/interface/assets.rs`), then
   `cite-reference`, update `ROADMAP.md`, and `check-and-commit`.
