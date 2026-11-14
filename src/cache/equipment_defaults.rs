//! The equipment defaults file (index 28, file 6, `DefaultsFile.EQUIPMENT`). Port of
//! `EquipmentDefaults.decode`: the slot table (opcode 1) is kept; the offhand and weapon slots
//! (3, 4) and the slots hidden by their animations (5, 6) are read past. An opcode it doesn't
//! know is skipped on its own, as the reference's `if` chain does.

use super::buffer::Reader;
use super::error::{corrupt, Result};

/// `hidden`: one byte per equipment slot; a slot whose byte is 1 is never in a player's
/// appearance (`PlayerEntity.decodeAppearance` reads a value for each of the others).
pub fn hidden_slots(data: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(data);
    let mut hidden = None;
    loop {
        match r.u8()? {
            0 => break,
            1 => {
                let count = usize::from(r.u8()?);
                hidden = Some(r.bytes(count)?.to_vec());
            }
            3 | 4 => {
                r.u8()?;
            }
            5 | 6 => {
                let count = usize::from(r.u8()?);
                r.bytes(count)?;
            }
            _ => {}
        }
    }
    // The constructor throws when the file has no slot table.
    match hidden {
        Some(h) => Ok(h),
        None => corrupt("the equipment defaults have no slot table"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slot_table_is_found_among_the_others() {
        assert_eq!(hidden_slots(&[3, 5, 1, 3, 0, 1, 0, 5, 2, 9, 9, 0]).unwrap(), vec![0, 1, 0]);
        assert!(hidden_slots(&[4, 3, 0]).is_err());
    }
}
