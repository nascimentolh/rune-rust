//! Client → server game packets: id and size of each (`ClientProt.java`, copied in id order).
//! A size of -1 means a u8 length comes first, -2 a u16 length. The id is one byte, added to
//! the client's ISAAC cipher (`writeIsaacByte`; the cipher is the login seeds).
//!
//! Generated from the reference's enum; names are its own (some obfuscated).

use super::isaac::Isaac;

/// (id, name, size).
pub const PACKETS: [(u32, &str, i32); 104] = [
    (0, "PING", 0),
    (1, "PLAYER_OP6", 3),
    (2, "SOUND_EFFECT_MUSIC_ENDED", 4),
    (3, "NPC_EXAMINE", 3),
    (4, "IF_ON_IF", 16),
    (5, "WORLD_MAP_CLICK", 4),
    (6, "PLAYER_OP2", 3),
    (7, "FC_SET_RANK", -1),
    (8, "GROUND_ITEM_OP4", 7),
    (9, "IF_OP4", 8),
    (10, "SEND_PREFERENCES", -1),
    (11, "RESUME_HSLDIALOG", 2),
    (12, "REMOVE_IGNORE", -1),
    (13, "IF_ON_PLAYER", 11),
    (14, "QUICKCHAT_PRIVATE", -1),
    (15, "SEND_PRIVATE_MESSAGE", -2),
    (16, "NPC_OP2", 3),
    (17, "GRAND_EXCHANGE_ITEM_SELECT", 2),
    (18, "SONG_LOADED", 4),
    (19, "IF_OP6", 8),
    (20, "CHAT_SETFILTER", 3),
    (21, "IF_OP8", 8),
    (22, "IF_OP9", 8),
    (23, "IF_OP7", 8),
    (24, "GROUND_ITEM_OP1", 7),
    (25, "GROUND_ITEM_OP2", 7),
    (26, "ADD_FRIEND", -1),
    (27, "IF_OP2", 8),
    (28, "KEY_PRESS", -2),
    (29, "REMOVE_FRIEND", -1),
    (30, "CHAT_TYPE", 1),
    (31, "PLAYER_OP3", 3),
    (32, "OBJECT_OP4", 9),
    (33, "WALK", 5),
    (34, "ADD_IGNORE", -1),
    (35, "BUG_REPORT", -2),
    (36, "REFLECTION_CHECK", -1),
    (37, "UNK_37", 2),
    (38, "OBJECT_OP3", 9),
    (39, "UNUSED_CLAN_OP", -1),
    (40, "MOVE_MOUSE", -1),
    (41, "IF_ON_NPC", 11),
    (42, "MINI_WALK", 18),
    (43, "GROUND_ITEM_OP5", 7),
    (44, "SEND_FPS", 9),
    (45, "CUTSCENE_FINISHED", 1),
    (46, "IF_ON_TILE", 12),
    (47, "REQUEST_WORLD_LIST", 4),
    (48, "OBJECT_OP5", 9),
    (49, "IF_CONTINUE", 6),
    (50, "NPC_OP3", 3),
    (51, "PLAYER_OP7", 3),
    (52, "EMAIL_VALIDATION_SUBMIT_CODE", -1),
    (53, "PLAYER_OP9", 3),
    (54, "GROUND_ITEM_OP3", 7),
    (55, "TRANSMITVAR_VERIFYID", 4),
    (56, "LOBBY_HYPERLINK", -2),
    (57, "MOUSE_BUTTON_CLICK", 7),
    (58, "RESUME_COUNTDIALOG", 4),
    (59, "MOUSE_CLICK", 6),
    (60, "CLOSE_INTERFACE", 0),
    (61, "GROUND_ITEM_EXAMINE", 7),
    (62, "CLIENT_FOCUS", 1),
    (63, "UNK_63", 4),
    (64, "QUICKCHAT_PUBLIC", -1),
    (65, "NPC_OP1", 3),
    (66, "PLAYER_OP1", 3),
    (67, "IF_ON_GROUND_ITEM", 15),
    (68, "IF_OP3", 8),
    (69, "RESUME_CLANFORUMQFCDIALOG", -1),
    (70, "PLAYER_OP10", 3),
    (71, "FC_JOIN", -1),
    (72, "IF_OP5", 8),
    (73, "OBJECT_EXAMINE", 9),
    (74, "IF_DRAG_ONTO_IF", 16),
    (75, "OBJECT_OP1", 9),
    (76, "REGION_LOADED_CONFIRM", 0),
    (77, "NPC_OP4", 3),
    (78, "MOVE_MOUSE_2", -1),
    (79, "UNK_79", 1),
    (80, "RESUME_NAMEDIALOG", -1),
    (81, "IF_OP10", 8),
    (82, "UNK_82", 4),
    (83, "MOVE_CAMERA", 4),
    (84, "SCREEN_SIZE", 6),
    (85, "COMMAND", -1),
    (86, "CHAT", -1),
    (87, "RESUME_TEXTDIALOG", -1),
    (88, "WRITE_PING", 2),
    (89, "PLAYER_OP4", 3),
    (90, "CLANCHANNEL_KICKUSER", -1),
    (91, "FC_KICK", -1),
    (92, "EMAIL_VALIDATION_ADD_NEW_ADDRESS", -2),
    (93, "OBJECT_OP2", 9),
    (94, "PLAYER_OP8", 3),
    (95, "NPC_OP5", 3),
    (96, "IF_OP1", 8),
    (97, "DEPRECATED_97_SERVERPACKET_52", -1),
    (98, "IF_ON_OBJECT", 17),
    (99, "EMAIL_VALIDATION_CHANGE_ADDRESS", -2),
    (100, "REPORT_ABUSE", -1),
    (101, "SEND_SIGN_UP_FORM", -2),
    (102, "CHECK_EMAIL_VALIDITY", -2),
    (103, "PLAYER_OP5", 3),
];

/// The size of packet `id` (-1 / -2 for a length that comes first), if known.
pub fn size_of(id: u32) -> Option<i32> {
    PACKETS.iter().find(|p| p.0 == id).map(|p| p.2)
}

/// The name of packet `id`, for logs.
pub fn name_of(id: u32) -> &'static str {
    PACKETS.iter().find(|p| p.0 == id).map_or("unknown", |p| p.1)
}

/// `ClientProt.REQUEST_WORLD_LIST`: the lobby asks for the world list (`crate::protocol::world_list`).
pub const REQUEST_WORLD_LIST: u32 = 47;

/// `IF_OP1` … `IF_OP10`: a component option was clicked (`CutsceneAction_Sub10.method14603`).
pub const IF_OPS: [u32; 10] = [96, 27, 68, 9, 72, 19, 23, 21, 22, 81];

/// The `IF_OP` payload (`Class350_Sub3.method12590`): the component's full id as a
/// little-endian int, its item (`containerItemId`, -1 for none) as a u16, its slot as a u16
/// with 128 added to the low byte (`writeShort128`).
pub fn if_op_payload(component: i32, slot: i32, item: i32) -> Vec<u8> {
    let mut out = component.to_le_bytes().to_vec();
    out.extend((item as u16).to_be_bytes());
    out.extend([(slot >> 8) as u8, (slot as u8).wrapping_add(128)]);
    out
}

/// A packet on the wire: the id plus the cipher's next value, the length for sizes -1 / -2,
/// then the payload.
pub fn frame(id: u32, payload: &[u8], cipher: &mut Isaac) -> Vec<u8> {
    let mut out = vec![(id as u8).wrapping_add(cipher.next() as u8)];
    match size_of(id) {
        Some(-1) => out.push(payload.len() as u8),
        Some(-2) => out.extend((payload.len() as u16).to_be_bytes()),
        _ => {}
    }
    out.extend(payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_op_matches_the_reference() {
        assert_eq!((IF_OPS[0], size_of(IF_OPS[0])), (96, Some(8)));
        assert_eq!(if_op_payload(906 << 16 | 215, -1, -1), vec![215, 0, 0x8a, 3, 0xff, 0xff, 0xff, 0x7f]);
    }
}
