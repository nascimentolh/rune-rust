//! `docs/PROTOCOL.md`, written from the two packet tables so it can't drift from the code:
//! `cargo run -- --protocol-doc` writes it, and a test fails while the file is out of date.

use super::client_prot::ClientProt;
use super::packet::Size;
use super::server_prot::ServerProt;

/// Where the file lives, from the project folder.
pub const FILE: &str = "docs/PROTOCOL.md";

/// One table row's facts, whichever table it is from.
struct Row {
    id: u32,
    name: &'static str,
    variant: String,
    size: Size,
    doc: &'static str,
}

fn size_text(size: Size) -> String {
    match size {
        Size::Fixed(n) => n.to_string(),
        Size::Byte => "u8 length".into(),
        Size::Short => "u16 length".into(),
    }
}

fn section(out: &mut String, title: &str, table: &str, rows: &[Row]) {
    let ported = rows.iter().filter(|r| !r.doc.is_empty()).count();
    out.push_str(&format!("## {title}\n\n`{table}`: {} packets, {ported} ported.\n\n", rows.len()));
    out.push_str("| Id | Name | In code | Size | Ported |\n|---:|---|---|---|---|\n");
    for r in rows {
        let ported = if r.doc.is_empty() { "" } else { "yes" };
        out.push_str(&format!("| {} | `{}` | `{table}::{}` | {} | {ported} |\n", r.id, r.name, r.variant, size_text(r.size)));
    }
    out.push_str(&format!("\n### Ported: {}\n\n", title.to_lowercase()));
    for r in rows.iter().filter(|r| !r.doc.is_empty()) {
        // A doc comment's lines each start with the space after `///`.
        let text: Vec<&str> = r.doc.lines().map(str::trim).collect();
        out.push_str(&format!("**{} `{}`** ({})\n\n{}\n\n", r.id, r.name, size_text(r.size), text.join("\n")));
    }
}

/// The whole file.
pub fn markdown() -> String {
    let mut out = String::from(
        "# Game protocol (build 727)\n\n\
         Written by `cargo run -- --protocol-doc` from `src/protocol/server_prot.rs` and\n\
         `src/protocol/client_prot.rs`. Don't edit it: change the tables' rows and doc comments and\n\
         run the command again (`cargo test` fails while this file is out of date).\n\n\
         Ids, names and sizes are the reference's (`ServerProt.java`, `ClientProt.java`). A packet\n\
         is ported when its row has a doc comment saying what its payload is and who uses it; how\n\
         to port one is in `docs/SERVER.md`.\n\n\
         On the wire, after login: a server packet's id is a smart (one byte below 128, else two)\n\
         with each byte added to the server's ISAAC cipher; a client packet's id is one byte added\n\
         to the client's cipher. A \"u8 length\" or \"u16 length\" size means that length comes next;\n\
         then the payload.\n\n",
    );
    let server: Vec<Row> = ServerProt::ALL
        .iter()
        .map(|p| Row { id: p.id(), name: p.name(), variant: format!("{p:?}"), size: p.size(), doc: p.doc() })
        .collect();
    let client: Vec<Row> = ClientProt::ALL
        .iter()
        .map(|p| Row { id: p.id(), name: p.name(), variant: format!("{p:?}"), size: p.size(), doc: p.doc() })
        .collect();
    section(&mut out, "Server → client", "ServerProt", &server);
    section(&mut out, "Client → server", "ClientProt", &client);
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}

/// `--protocol-doc`: writes the file into the current folder's `docs/`.
pub fn write() -> Result<(), String> {
    std::fs::write(FILE, markdown()).map_err(|e| format!("can't write {FILE}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_protocol_document_is_current() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE);
        let on_disk = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
        assert!(on_disk == markdown(), "{FILE} is out of date: run `cargo run -- --protocol-doc`");
    }
}
