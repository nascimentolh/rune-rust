//! Server → client game packets: id and size of each (`ServerProt.java`, copied in id order).
//! A size of -1 means a u8 length comes first, -2 a u16 length.
//!
//! Generated from the reference's enum; names are its own (some obfuscated).

use super::isaac::Isaac;

/// (id, name, size).
pub const PACKETS: [(u32, &str, i32); 162] = [
    (0, "IF_SETPLAYERHEAD", 4),
    (1, "CREATE_CHECK_EMAIL_REPLY", 1),
    (2, "PROCESS_DEV_CONSOLE_COMMAND", -1),
    (3, "GROUND_ITEM_COUNT", 7),
    (4, "UPDATE_INV_PARTIAL", -2),
    (5, "UPDATE_INV_FULL", -2),
    (6, "NPC_UPDATE", -2),
    (7, "IF_SETTEXTFONT", 8),
    (8, "VARP_LARGE", 6),
    (9, "LOGOUT_LOBBY", 0),
    (10, "MESSAGE_PRIVATE_ECHO", -2),
    (11, "aClass375_4362", 2),
    (12, "CLIENT_SETVARC_LARGE", 6),
    (13, "IF_MOVESUB", 8),
    (14, "PLAYER_WEIGHT", 2),
    (15, "UPDATE_ZONE_PARTIAL_FOLLOWS", 3),
    (16, "OPEN_URL", -2),
    (17, "aClass375_4368", 3),
    (18, "VORBIS_SOUND", 8),
    (19, "UPDATE_FRIENDCHAT_CHANNEL_SINGLEUSER", -1),
    (20, "OUTDATED_SET_THEORA_STRING_SOMETHING", 6),
    (21, "SHOW_FACE_HERE", 1),
    (22, "MESSAGE_QUICKCHAT_PRIVATE", -1),
    (23, "IF_OPENSUB_ACTIVE_OBJECT", 32),
    (24, "CAM_LOOKAT", 6),
    (25, "MESSAGE_FRIENDS_CHAT", -1),
    (26, "VARCLAN_SET_LONG", 10),
    (27, "CREATE_GROUND_ITEM", 5),
    (28, "CLANCHANNEL_DELTA", -2),
    (29, "PING", 0),
    (30, "CHAT_FILTER_SETTINGS", 2),
    (31, "MESSAGE_QUICKCHAT_PRIVATE_ECHO", -1),
    (32, "LOYALTY_UPDATE", 5),
    (33, "IF_SETHIDE", 5),
    (34, "IF_OPENSUB_ACTIVE_PLAYER", 25),
    (35, "IF_SETGRAPHIC", 8),
    (36, "IF_SETTEXTANTIMACRO", 5),
    (37, "IF_OPENTOP", 19),
    (38, "IF_OPENSUB", 23),
    (39, "CAM_MOVETO", 6),
    (40, "aClass375_4399", 4),
    (41, "UPDATE_ZONE_FULL_FOLLOWS", 3),
    (42, "UPDATE_INV_STOP_TRANSMIT", 3),
    (43, "PLAYER_UPDATE", -2),
    (44, "IF_SETANGLE", 10),
    (45, "VARCLAN_ENABLE", 0),
    (46, "CLANSETTINGS_DELTA", -2),
    (47, "NPC_UPDATE_LARGE", -2),
    (48, "IF_SETPOSITION", 8),
    (49, "CLANCHANNEL_FULL", -2),
    (50, "SET_CLAN_STRING", -1),
    (51, "DYNAMIC_MAP_REGION", -2),
    (52, "DEPRECATED_52_CLIENTPACKET_97", 1),
    (53, "IF_SETPLAYERMODEL_OTHER", 10),
    (54, "CLIENT_SETVARCSTR_SMALL", -1),
    (55, "IF_SETSCROLLPOS", 6),
    (56, "PROJANIM_SPECIFIC", 22),
    (57, "UPDATE_GE_SLOT", 20),
    (58, "QUICK_HOP_WORLDS", -1),
    (59, "IF_OPENSUB_ACTIVE_NPC", 25),
    (60, "MUSIC_EFFECT", 6),
    (61, "REDUCE_ATTACK_PRIORITY", 1),
    (62, "LOGOUT_FULL", 0),
    (63, "MUSIC_TRACK", 4),
    (64, "RUN_ENERGY", 1),
    (65, "UPDATE_ZONE_PARTIAL_ENCLOSED", -2),
    (66, "CAM_SMOOTHRESET", 0),
    (67, "CHAT_FILTER_SETTINGS_PRIVATECHAT", 1),
    (68, "VARBIT_SMALL", 3),
    (69, "DESTROY_OBJECT", 2),
    (70, "MINIMAP_FLAG", 2),
    (71, "CAM_SHAKE", 6),
    (72, "IF_SETPLAYERMODEL", 4),
    (73, "DEBUG_SERVER_TRIGGERS", -1),
    (74, "FRIEND_STATUS", -2),
    (75, "JCOINS_UPDATE", 4),
    (76, "APPLY_DEBUG", 2),
    (77, "PRELOAD_SONG", 2),
    (78, "IF_CLOSESUB", 4),
    (79, "HINT_ARROW", 14),
    (80, "OBJ_ANIM_SPECIFIC", 9),
    (81, "MESSAGE_CLANCHANNEL", -1),
    (82, "IF_SETANIM", 8),
    (83, "MESSAGE_QUICKCHAT_PLAYER_GROUP", -1),
    (84, "CUSTOMIZE_OBJECT", -1),
    (85, "REGION", -2),
    (86, "aClass375_4437", -2),
    (87, "CREATE_ACCOUNT_REPLY", 1),
    (88, "OBJECT_PREFETCH", 5),
    (89, "CAM_RESET", 0),
    (90, "UPDATE_UID192", 28),
    (91, "UPDATE_SITESETTINGS_COOKIE", -1),
    (92, "SEND_PRIVATE_MESSAGE", -2),
    (93, "aClass375_3828", 11),
    (94, "VARCLAN_DISABLE", 0),
    (95, "MAP_PROJANIM_HALFSQ", 17),
    (96, "OUTDATED_INTERFACE_PULSE_EVENT", 8),
    (97, "UPDATE_IGNORE_LIST", -2),
    (98, "REFLECTION_CHECK", -2),
    (99, "RUN_CS2_SCRIPT", -2),
    (100, "CLEAR_VARPS", 0),
    (101, "FRIENDLIST_LOADED", 0),
    (102, "UPDATE_REBOOT_TIMER", 2),
    (103, "WORLD_LIST", -2),
    (104, "VORBIS_SPEECH_SOUND", 6),
    (105, "IF_SETCOLOR", 6),
    (106, "aClass375_4457", 2),
    (107, "GROUND_ITEM_REVEAL", 7),
    (108, "VARBIT_LARGE", 6),
    (109, "IF_SETNPCHEAD", 8),
    (110, "CUTSCENE", -2),
    (111, "PLAYER_OPTION", -1),
    (112, "IF_SETITEM", 10),
    (113, "OBJ_ANIM", 6),
    (114, "TILE_MESSAGE", -1),
    (115, "VARP_SMALL", 3),
    (116, "CLIENT_SETVARC_SMALL", 3),
    (117, "CREATE_OBJECT", 6),
    (118, "CAM_FORCEANGLE", 4),
    (119, "CLIENT_SETVARCSTR_LARGE", -2),
    (120, "RESET_SOUNDS", 0),
    (121, "IF_SETTARGETPARAM", 12),
    (122, "RESET_ALL_ANIMATIONS", 0),
    (123, "VARCLAN_SET_BYTE", 3),
    (124, "IF_SETTEXT", -2),
    (125, "REMOVE_GROUND_ITEM", 3),
    (126, "SPOT_ANIM_SPECIFIC", 12),
    (127, "FRIENDS_CHAT_CHANNEL", -2),
    (128, "MIDI_SONG_LOCATION", 6),
    (129, "SET_TARGET", 2),
    (130, "IF_SETPLAYERHEAD_IGNOREWORN", 10),
    (131, "MESSAGE_QUICKCHAT_CLANCHANNEL", -1),
    (132, "IF_SETRETEX", 9),
    (133, "MESSAGE_PLAYER_GROUP", -1),
    (134, "aClass375_4453", 2),
    (135, "IF_SETMODEL", 8),
    (136, "SOUND_SYNTH", 8),
    (137, "CLANSETTINGS_FULL", -2),
    (138, "ADD_IGNORE", -1),
    (139, "SPOT_ANIM", 8),
    (140, "UPDATE_STAT", 6),
    (141, "VARCLAN_SET_INT", 6),
    (142, "MESSAGE_QUICKCHAT_FRIENDS_CHAT", -1),
    (143, "MAP_PROJANIM", 16),
    (144, "IF_SETRECOL", 9),
    (145, "ANIMATE_NPC", 19),
    (146, "aClass375_4497", 0),
    (147, "IDENTIFY_HOST_NAME", 4),
    (148, "IF_RESETTARGETPARAM", 10),
    (149, "HINT_TRAIL", -2),
    (150, "IF_SETCLICKMASK", 5),
    (151, "SET_DRAW_ORDER", 1),
    (152, "MESSAGE_PUBLIC", -1),
    (153, "SET_CURSOR", -1),
    (154, "BLOCK_MINIMAP_STATE", 1),
    (155, "IF_OPENSUB_ACTIVE_GROUNDITEM", 29),
    (156, "aClass375_4507", -2),
    (157, "UPDATE_DOB", 4),
    (158, "IF_SETPLAYERHEAD_OTHER", 10),
    (159, "REQUEST_FPS", 8),
    (160, "GAME_MESSAGE", -1),
    (161, "DISCORD_RICH_PRESENCE_UPDATE", -1),
];

/// The size of packet `id` (-1 / -2 for a length that comes first), if the client knows it.
pub fn size_of(id: u32) -> Option<i32> {
    PACKETS.iter().find(|p| p.0 == id).map(|p| p.2)
}

/// `ServerProt.REGION`: the first game packet of a world login (player positions, map scene).
pub const REGION: u32 = 85;
/// `ServerProt.VARP_SMALL`: a player variable 0..=255 (value i8, then id u16 LE+128).
pub const VARP_SMALL: u32 = 115;
/// `ServerProt.VARP_LARGE`: a player variable (value as an int in V2 order, then id u16 LE+128).
pub const VARP_LARGE: u32 = 8;
/// `ServerProt.WORLD_LIST`: the world list or its player counts (`crate::protocol::world_list`).
pub const WORLD_LIST: u32 = 103;
/// `ServerProt.IF_OPENTOP`: the top-level window (`PacketDecoder`: key int V1, flags int V1,
/// window id u16 +128, type 128 − byte, two ints).
pub const IF_OPENTOP: u32 = 37;
/// `ServerProt.IF_OPENSUB`: an interface opened in a component (key int, flags int V2, an int
/// V2, the interface u16 LE +128, an int LE, the parent component int, the type −byte).
pub const IF_OPENSUB: u32 = 38;

/// The `IF_OPENTOP` payload for window `id` (keys and type 0).
pub fn open_top(id: u16) -> Vec<u8> {
    let mut out = vec![0u8; 8];
    out.extend([(id >> 8) as u8, (id as u8).wrapping_add(128)]);
    out.push(128);
    out.extend([0u8; 8]);
    out
}

/// A packet on the wire: the id as an ISAAC-scrambled smart (`readEncryptedSmart`), then its
/// length when its size is -1 (u8) or -2 (u16), then the payload.
pub fn frame(id: u32, payload: &[u8], cipher: &mut Isaac) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 4);
    if id < 128 {
        out.push((id as u8).wrapping_add(cipher.next() as u8));
    } else {
        out.push(((id >> 8) as u8 + 128).wrapping_add(cipher.next() as u8));
        out.push((id as u8).wrapping_add(cipher.next() as u8));
    }
    match size_of(id) {
        Some(-1) => out.push(payload.len() as u8),
        Some(-2) => out.extend((payload.len() as u16).to_be_bytes()),
        _ => {}
    }
    out.extend(payload);
    out
}

/// A player variable as `PacketDecoder` reads it: `VARP_SMALL` for 0..=255
/// (`readByte`, `readUnsignedShortLE128`), else `VARP_LARGE` (`readIntV2`, then the id).
pub fn varp(id: u16, value: i32) -> (u32, Vec<u8>) {
    let id_le128 = [(id as u8).wrapping_add(128), (id >> 8) as u8];
    if (0..=255).contains(&value) {
        (VARP_SMALL, vec![value as u8, id_le128[0], id_le128[1]])
    } else {
        let v = value.to_be_bytes();
        // `readIntV2`: bytes in the order 2, 1, 4, 3 of the big-endian int.
        (VARP_LARGE, vec![v[1], v[0], v[3], v[2], id_le128[0], id_le128[1]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_match_the_reference() {
        assert_eq!(size_of(VARP_SMALL), Some(3));
        assert_eq!(size_of(VARP_LARGE), Some(6));
        assert_eq!(size_of(85), Some(-2)); // REGION
    }
}
