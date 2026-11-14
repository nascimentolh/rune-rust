//! What the two packet tables (`ServerProt`, `ClientProt`) are made of: a packet's size, and
//! the macro that turns a table copied from the reference's enum into a typed Rust enum.
//!
//! A table row is `Variant = id, "REFERENCE_NAME", size;` in id order, the size as the reference
//! writes it (-1 = a u8 length comes first, -2 = a u16 length). A row gets a doc comment when the
//! packet is ported: its payload as the reference client reads or writes it, which function
//! here encodes or decodes it, and who sends or handles it. A row without one is not ported.
//! `docs/PROTOCOL.md` is written from these tables (`cargo run -- --protocol-doc`).

/// How long a packet's payload is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// Always this many bytes.
    Fixed(usize),
    /// A u8 length comes first (the reference's -1).
    Byte,
    /// A u16 length comes first (the reference's -2).
    Short,
}

impl Size {
    /// The size as `ServerProt.java` / `ClientProt.java` give it.
    pub const fn from_reference(size: i32) -> Self {
        match size {
            -1 => Size::Byte,
            -2 => Size::Short,
            n => Size::Fixed(n as usize),
        }
    }
}

/// Builds a packet enum from a table in id order (ids 0, 1, 2… with none missing).
macro_rules! packet_table {
    ($(#[$meta:meta])* $name:ident { $($(#[doc = $doc:literal])* $variant:ident = $id:literal, $text:literal, $size:literal;)* }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($(#[doc = $doc])* $variant = $id,)*
        }

        impl $name {
            /// Every packet, in id order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            /// The id on the wire.
            pub const fn id(self) -> u32 {
                self as u32
            }

            /// The packet with this id, if the reference has one.
            pub fn from_id(id: u32) -> Option<Self> {
                Self::ALL.get(id as usize).copied()
            }

            /// The reference's name for it.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)*
                }
            }

            pub const fn size(self) -> $crate::protocol::packet::Size {
                match self {
                    $(Self::$variant => $crate::protocol::packet::Size::from_reference($size),)*
                }
            }

            /// Its payload and who uses it (the row's doc comment); empty while it isn't ported.
            pub const fn doc(self) -> &'static str {
                match self {
                    $(Self::$variant => concat!($($doc, "\n",)* ""),)*
                }
            }
        }
    };
}

pub(crate) use packet_table;

#[cfg(test)]
mod tests {
    use crate::protocol::client_prot::ClientProt;
    use crate::protocol::server_prot::ServerProt;

    /// `from_id` indexes the table, so each row must sit at its id; the counts are the reference's.
    #[test]
    fn both_tables_are_whole_and_in_id_order() {
        assert_eq!((ServerProt::ALL.len(), ClientProt::ALL.len()), (162, 104));
        assert!(ServerProt::ALL.iter().enumerate().all(|(n, p)| p.id() as usize == n));
        assert!(ClientProt::ALL.iter().enumerate().all(|(n, p)| p.id() as usize == n));
        assert_eq!(ServerProt::from_id(162), None);
    }
}
