//! Client → server game packets: the id, name and size of each (`ClientProt.java`, copied in id
//! order), and how one is framed on the wire. Their payloads are built and read in
//! `client_packets`.
//!
//! The names are the reference's own. How rows are written and documented: `super::packet`.

use super::isaac::Isaac;
use super::packet::{packet_table, Size};

packet_table! {
    /// A client → server game packet.
    ClientProt {
        Ping = 0, "PING", 0;
        PlayerOp6 = 1, "PLAYER_OP6", 3;
        SoundEffectMusicEnded = 2, "SOUND_EFFECT_MUSIC_ENDED", 4;
        NpcExamine = 3, "NPC_EXAMINE", 3;
        IfOnIf = 4, "IF_ON_IF", 16;
        WorldMapClick = 5, "WORLD_MAP_CLICK", 4;
        PlayerOp2 = 6, "PLAYER_OP2", 3;
        FcSetRank = 7, "FC_SET_RANK", -1;
        GroundItemOp4 = 8, "GROUND_ITEM_OP4", 7;
        /// Option 4 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp4 = 9, "IF_OP4", 8;
        SendPreferences = 10, "SEND_PREFERENCES", -1;
        ResumeHsldialog = 11, "RESUME_HSLDIALOG", 2;
        RemoveIgnore = 12, "REMOVE_IGNORE", -1;
        IfOnPlayer = 13, "IF_ON_PLAYER", 11;
        QuickchatPrivate = 14, "QUICKCHAT_PRIVATE", -1;
        SendPrivateMessage = 15, "SEND_PRIVATE_MESSAGE", -2;
        NpcOp2 = 16, "NPC_OP2", 3;
        GrandExchangeItemSelect = 17, "GRAND_EXCHANGE_ITEM_SELECT", 2;
        SongLoaded = 18, "SONG_LOADED", 4;
        /// Option 6 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp6 = 19, "IF_OP6", 8;
        ChatSetfilter = 20, "CHAT_SETFILTER", 3;
        /// Option 8 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp8 = 21, "IF_OP8", 8;
        /// Option 9 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp9 = 22, "IF_OP9", 8;
        /// Option 7 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp7 = 23, "IF_OP7", 8;
        GroundItemOp1 = 24, "GROUND_ITEM_OP1", 7;
        GroundItemOp2 = 25, "GROUND_ITEM_OP2", 7;
        AddFriend = 26, "ADD_FRIEND", -1;
        /// Option 2 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp2 = 27, "IF_OP2", 8;
        KeyPress = 28, "KEY_PRESS", -2;
        RemoveFriend = 29, "REMOVE_FRIEND", -1;
        ChatType = 30, "CHAT_TYPE", 1;
        PlayerOp3 = 31, "PLAYER_OP3", 3;
        ObjectOp4 = 32, "OBJECT_OP4", 9;
        Walk = 33, "WALK", 5;
        AddIgnore = 34, "ADD_IGNORE", -1;
        BugReport = 35, "BUG_REPORT", -2;
        ReflectionCheck = 36, "REFLECTION_CHECK", -1;
        Unk37 = 37, "UNK_37", 2;
        ObjectOp3 = 38, "OBJECT_OP3", 9;
        UnusedClanOp = 39, "UNUSED_CLAN_OP", -1;
        MoveMouse = 40, "MOVE_MOUSE", -1;
        IfOnNpc = 41, "IF_ON_NPC", 11;
        MiniWalk = 42, "MINI_WALK", 18;
        GroundItemOp5 = 43, "GROUND_ITEM_OP5", 7;
        SendFps = 44, "SEND_FPS", 9;
        CutsceneFinished = 45, "CUTSCENE_FINISHED", 1;
        IfOnTile = 46, "IF_ON_TILE", 12;
        /// The lobby asks for the world list: the checksum (int) of the list it holds, 0 for none.
        /// Encoder and decoder: `client_packets::request_world_list`, `client_packets::world_list_held`.
        /// Handled by the lobby (`server::lobby`), which answers with `WORLD_LIST`.
        RequestWorldList = 47, "REQUEST_WORLD_LIST", 4;
        ObjectOp5 = 48, "OBJECT_OP5", 9;
        IfContinue = 49, "IF_CONTINUE", 6;
        NpcOp3 = 50, "NPC_OP3", 3;
        PlayerOp7 = 51, "PLAYER_OP7", 3;
        EmailValidationSubmitCode = 52, "EMAIL_VALIDATION_SUBMIT_CODE", -1;
        PlayerOp9 = 53, "PLAYER_OP9", 3;
        GroundItemOp3 = 54, "GROUND_ITEM_OP3", 7;
        TransmitvarVerifyid = 55, "TRANSMITVAR_VERIFYID", 4;
        LobbyHyperlink = 56, "LOBBY_HYPERLINK", -2;
        MouseButtonClick = 57, "MOUSE_BUTTON_CLICK", 7;
        ResumeCountdialog = 58, "RESUME_COUNTDIALOG", 4;
        MouseClick = 59, "MOUSE_CLICK", 6;
        CloseInterface = 60, "CLOSE_INTERFACE", 0;
        GroundItemExamine = 61, "GROUND_ITEM_EXAMINE", 7;
        ClientFocus = 62, "CLIENT_FOCUS", 1;
        Unk63 = 63, "UNK_63", 4;
        QuickchatPublic = 64, "QUICKCHAT_PUBLIC", -1;
        NpcOp1 = 65, "NPC_OP1", 3;
        PlayerOp1 = 66, "PLAYER_OP1", 3;
        IfOnGroundItem = 67, "IF_ON_GROUND_ITEM", 15;
        /// Option 3 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp3 = 68, "IF_OP3", 8;
        ResumeClanforumqfcdialog = 69, "RESUME_CLANFORUMQFCDIALOG", -1;
        PlayerOp10 = 70, "PLAYER_OP10", 3;
        FcJoin = 71, "FC_JOIN", -1;
        /// Option 5 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp5 = 72, "IF_OP5", 8;
        ObjectExamine = 73, "OBJECT_EXAMINE", 9;
        IfDragOntoIf = 74, "IF_DRAG_ONTO_IF", 16;
        ObjectOp1 = 75, "OBJECT_OP1", 9;
        RegionLoadedConfirm = 76, "REGION_LOADED_CONFIRM", 0;
        NpcOp4 = 77, "NPC_OP4", 3;
        MoveMouse2 = 78, "MOVE_MOUSE_2", -1;
        Unk79 = 79, "UNK_79", 1;
        ResumeNamedialog = 80, "RESUME_NAMEDIALOG", -1;
        /// Option 10 of a component was clicked: the same body as `IF_OP1`. Sent by the client
        /// (`client_packets::if_op`); the server has no handler for it yet.
        IfOp10 = 81, "IF_OP10", 8;
        Unk82 = 82, "UNK_82", 4;
        MoveCamera = 83, "MOVE_CAMERA", 4;
        /// The window mode or game area changed while in the game (`Class388.method6692`): the mode
        /// (`Class158.windowedMode`: 1 fixed, 2 resizable, 3 full screen), the game area's width and
        /// height (u16 each), the anti-aliasing preference (byte). Encoder and decoder:
        /// `client_packets::screen_size`, `client_packets::screen_size_mode`. Handled by the world
        /// (`server::world::handlers::interface`), which opens the other game window when the mode needs it.
        ScreenSize = 84, "SCREEN_SIZE", 6;
        Command = 85, "COMMAND", -1;
        Chat = 86, "CHAT", -1;
        ResumeTextdialog = 87, "RESUME_TEXTDIALOG", -1;
        WritePing = 88, "WRITE_PING", 2;
        PlayerOp4 = 89, "PLAYER_OP4", 3;
        ClanchannelKickuser = 90, "CLANCHANNEL_KICKUSER", -1;
        FcKick = 91, "FC_KICK", -1;
        EmailValidationAddNewAddress = 92, "EMAIL_VALIDATION_ADD_NEW_ADDRESS", -2;
        ObjectOp2 = 93, "OBJECT_OP2", 9;
        PlayerOp8 = 94, "PLAYER_OP8", 3;
        NpcOp5 = 95, "NPC_OP5", 3;
        /// Option 1 of a component was clicked (`CutsceneAction_Sub10.method14603`,
        /// `Class350_Sub3.method12590`): the component (int LE: interface << 16 | component), its item
        /// (u16, 65535 for none), its slot (u16, low byte + 128). Encoder and decoder:
        /// `client_packets::if_op`, `client_packets::if_op_component`. Handled by the world
        /// (`server::world::handlers::interface`) for the logout tab's two buttons.
        IfOp1 = 96, "IF_OP1", 8;
        Deprecated97Serverpacket52 = 97, "DEPRECATED_97_SERVERPACKET_52", -1;
        IfOnObject = 98, "IF_ON_OBJECT", 17;
        EmailValidationChangeAddress = 99, "EMAIL_VALIDATION_CHANGE_ADDRESS", -2;
        ReportAbuse = 100, "REPORT_ABUSE", -1;
        SendSignUpForm = 101, "SEND_SIGN_UP_FORM", -2;
        CheckEmailValidity = 102, "CHECK_EMAIL_VALIDITY", -2;
        PlayerOp5 = 103, "PLAYER_OP5", 3;
    }
}

impl ClientProt {
    /// `IF_OP1` … `IF_OP10`: a component's option 1 to 10 was clicked.
    pub const IF_OPS: [ClientProt; 10] = [
        ClientProt::IfOp1,
        ClientProt::IfOp2,
        ClientProt::IfOp3,
        ClientProt::IfOp4,
        ClientProt::IfOp5,
        ClientProt::IfOp6,
        ClientProt::IfOp7,
        ClientProt::IfOp8,
        ClientProt::IfOp9,
        ClientProt::IfOp10,
    ];
}

/// A packet ready to send: which one, and its payload.
pub type Packet = (ClientProt, Vec<u8>);

/// A packet on the wire: the id plus the cipher's next value (`writeIsaacByte`; the cipher is
/// the login seeds), the length when its size is a u8 or a u16, then the payload.
pub fn frame(packet: ClientProt, payload: &[u8], cipher: &mut Isaac) -> Vec<u8> {
    let mut out = vec![(packet.id() as u8).wrapping_add(cipher.next() as u8)];
    match packet.size() {
        Size::Byte => out.push(payload.len() as u8),
        Size::Short => out.extend((payload.len() as u16).to_be_bytes()),
        Size::Fixed(_) => {}
    }
    out.extend(payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_option_packets_match_the_reference() {
        let ids: Vec<u32> = ClientProt::IF_OPS.iter().map(|p| p.id()).collect();
        assert_eq!(ids, [96, 27, 68, 9, 72, 19, 23, 21, 22, 81]);
        assert!(ClientProt::IF_OPS.iter().all(|p| p.size() == Size::Fixed(8)));
        assert_eq!(ClientProt::from_id(84), Some(ClientProt::ScreenSize));
    }
}
