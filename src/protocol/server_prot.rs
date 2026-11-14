//! Server → client game packets: the id, name and size of each (`ServerProt.java`, copied in id
//! order), and how one is framed on the wire. Their payloads are built in `server_packets`.
//!
//! The names are the reference's own; the ones it left obfuscated (`aClass375_…`) are
//! `Unnamed<id>` here. How rows are written and documented: `super::packet`.

use super::isaac::Isaac;
use super::packet::{packet_table, Size};

packet_table! {
    /// A server → client game packet.
    ServerProt {
        IfSetplayerhead = 0, "IF_SETPLAYERHEAD", 4;
        CreateCheckEmailReply = 1, "CREATE_CHECK_EMAIL_REPLY", 1;
        ProcessDevConsoleCommand = 2, "PROCESS_DEV_CONSOLE_COMMAND", -1;
        GroundItemCount = 3, "GROUND_ITEM_COUNT", 7;
        UpdateInvPartial = 4, "UPDATE_INV_PARTIAL", -2;
        UpdateInvFull = 5, "UPDATE_INV_FULL", -2;
        NpcUpdate = 6, "NPC_UPDATE", -2;
        IfSettextfont = 7, "IF_SETTEXTFONT", 8;
        /// A player variable whose value doesn't fit a signed byte: the value (`readIntV2`: bytes 2, 1,
        /// 4, 3 of the big-endian int), then the id (`readUnsignedShortLE128`: low byte + 128, then
        /// the high byte). Encoder: `server_packets::varp`. Sent by the lobby and the world.
        VarpLarge = 8, "VARP_LARGE", 6;
        /// No body. `CursorIndexLoader.logout(Class9.aBool71)`: back to the lobby when the player
        /// came through it, else to the login screen. Encoder: `server_packets::logout`. Sent by the
        /// world for the logout tab's "Exit to Lobby".
        LogoutLobby = 9, "LOGOUT_LOBBY", 0;
        MessagePrivateEcho = 10, "MESSAGE_PRIVATE_ECHO", -2;
        Unnamed11 = 11, "aClass375_4362", 2;
        /// A client variable whose value doesn't fit a signed byte: the id (`readShortLE`), then the
        /// value (`readIntV2`: bytes 2, 1, 4, 3). Encoder: `server_packets::varc`.
        ClientSetvarcLarge = 12, "CLIENT_SETVARC_LARGE", 6;
        IfMovesub = 13, "IF_MOVESUB", 8;
        PlayerWeight = 14, "PLAYER_WEIGHT", 2;
        UpdateZonePartialFollows = 15, "UPDATE_ZONE_PARTIAL_FOLLOWS", 3;
        OpenUrl = 16, "OPEN_URL", -2;
        Unnamed17 = 17, "aClass375_4368", 3;
        VorbisSound = 18, "VORBIS_SOUND", 8;
        UpdateFriendchatChannelSingleuser = 19, "UPDATE_FRIENDCHAT_CHANNEL_SINGLEUSER", -1;
        OutdatedSetTheoraStringSomething = 20, "OUTDATED_SET_THEORA_STRING_SOMETHING", 6;
        ShowFaceHere = 21, "SHOW_FACE_HERE", 1;
        MessageQuickchatPrivate = 22, "MESSAGE_QUICKCHAT_PRIVATE", -1;
        IfOpensubActiveObject = 23, "IF_OPENSUB_ACTIVE_OBJECT", 32;
        CamLookat = 24, "CAM_LOOKAT", 6;
        MessageFriendsChat = 25, "MESSAGE_FRIENDS_CHAT", -1;
        VarclanSetLong = 26, "VARCLAN_SET_LONG", 10;
        CreateGroundItem = 27, "CREATE_GROUND_ITEM", 5;
        ClanchannelDelta = 28, "CLANCHANNEL_DELTA", -2;
        Ping = 29, "PING", 0;
        ChatFilterSettings = 30, "CHAT_FILTER_SETTINGS", 2;
        MessageQuickchatPrivateEcho = 31, "MESSAGE_QUICKCHAT_PRIVATE_ECHO", -1;
        LoyaltyUpdate = 32, "LOYALTY_UPDATE", 5;
        IfSethide = 33, "IF_SETHIDE", 5;
        IfOpensubActivePlayer = 34, "IF_OPENSUB_ACTIVE_PLAYER", 25;
        IfSetgraphic = 35, "IF_SETGRAPHIC", 8;
        IfSettextantimacro = 36, "IF_SETTEXTANTIMACRO", 5;
        /// Opens the top-level window (`PacketDecoder`): a key int (V1), a flags int (V1), the window
        /// id (u16, low byte + 128), the type (128 − byte), two ints. This server sends the keys, the
        /// type and the ints as 0. Encoder: `server_packets::open_top`. Sent by the world on entering
        /// and when the window mode changes.
        IfOpentop = 37, "IF_OPENTOP", 19;
        /// Opens an interface in a component (`PacketDecoder`, `IdentiKitIndexLoader.openSub`): a key
        /// int, a flags int (V2), an int (V2), the interface (u16 LE, low byte + 128), an int (LE), the
        /// parent component (int: interface << 16 | component), the type (`readUnsignedByteC`: sent
        /// negated; 1 = an overlay). This server sends the keys and ints as 0. Encoder:
        /// `server_packets::open_sub`. Sent by the world for the side panels, the orbs and the chat.
        IfOpensub = 38, "IF_OPENSUB", 23;
        CamMoveto = 39, "CAM_MOVETO", 6;
        Unnamed40 = 40, "aClass375_4399", 4;
        UpdateZoneFullFollows = 41, "UPDATE_ZONE_FULL_FOLLOWS", 3;
        UpdateInvStopTransmit = 42, "UPDATE_INV_STOP_TRANSMIT", 3;
        PlayerUpdate = 43, "PLAYER_UPDATE", -2;
        IfSetangle = 44, "IF_SETANGLE", 10;
        VarclanEnable = 45, "VARCLAN_ENABLE", 0;
        ClansettingsDelta = 46, "CLANSETTINGS_DELTA", -2;
        NpcUpdateLarge = 47, "NPC_UPDATE_LARGE", -2;
        IfSetposition = 48, "IF_SETPOSITION", 8;
        ClanchannelFull = 49, "CLANCHANNEL_FULL", -2;
        SetClanString = 50, "SET_CLAN_STRING", -1;
        DynamicMapRegion = 51, "DYNAMIC_MAP_REGION", -2;
        Deprecated52Clientpacket97 = 52, "DEPRECATED_52_CLIENTPACKET_97", 1;
        IfSetplayermodelOther = 53, "IF_SETPLAYERMODEL_OTHER", 10;
        ClientSetvarcstrSmall = 54, "CLIENT_SETVARCSTR_SMALL", -1;
        IfSetscrollpos = 55, "IF_SETSCROLLPOS", 6;
        ProjanimSpecific = 56, "PROJANIM_SPECIFIC", 22;
        UpdateGeSlot = 57, "UPDATE_GE_SLOT", 20;
        QuickHopWorlds = 58, "QUICK_HOP_WORLDS", -1;
        IfOpensubActiveNpc = 59, "IF_OPENSUB_ACTIVE_NPC", 25;
        MusicEffect = 60, "MUSIC_EFFECT", 6;
        ReduceAttackPriority = 61, "REDUCE_ATTACK_PRIORITY", 1;
        /// No body. `CursorIndexLoader.logout(false)`: to the login screen. Encoder:
        /// `server_packets::logout`. Sent by the world for the logout tab's "Exit to Login".
        LogoutFull = 62, "LOGOUT_FULL", 0;
        MusicTrack = 63, "MUSIC_TRACK", 4;
        /// The run energy, one unsigned byte (0–100): `client.RUN_ENERGY`. Encoder:
        /// `server_packets::run_energy`. Sent by the world on entering.
        RunEnergy = 64, "RUN_ENERGY", 1;
        UpdateZonePartialEnclosed = 65, "UPDATE_ZONE_PARTIAL_ENCLOSED", -2;
        CamSmoothreset = 66, "CAM_SMOOTHRESET", 0;
        ChatFilterSettingsPrivatechat = 67, "CHAT_FILTER_SETTINGS_PRIVATECHAT", 1;
        VarbitSmall = 68, "VARBIT_SMALL", 3;
        DestroyObject = 69, "DESTROY_OBJECT", 2;
        MinimapFlag = 70, "MINIMAP_FLAG", 2;
        CamShake = 71, "CAM_SHAKE", 6;
        IfSetplayermodel = 72, "IF_SETPLAYERMODEL", 4;
        DebugServerTriggers = 73, "DEBUG_SERVER_TRIGGERS", -1;
        FriendStatus = 74, "FRIEND_STATUS", -2;
        JcoinsUpdate = 75, "JCOINS_UPDATE", 4;
        ApplyDebug = 76, "APPLY_DEBUG", 2;
        PreloadSong = 77, "PRELOAD_SONG", 2;
        IfClosesub = 78, "IF_CLOSESUB", 4;
        HintArrow = 79, "HINT_ARROW", 14;
        ObjAnimSpecific = 80, "OBJ_ANIM_SPECIFIC", 9;
        MessageClanchannel = 81, "MESSAGE_CLANCHANNEL", -1;
        IfSetanim = 82, "IF_SETANIM", 8;
        MessageQuickchatPlayerGroup = 83, "MESSAGE_QUICKCHAT_PLAYER_GROUP", -1;
        CustomizeObject = 84, "CUSTOMIZE_OBJECT", -1;
        /// The first game packet of a world login (`Login.java`, stages 167 → 173 → 156): every
        /// player's position in bits (`Class197.method3203`), then the map scene
        /// (`MapRegion.loadMapSceneNormal`) with the XTEA keys of the map regions around. Layout and
        /// encoder: `crate::protocol::region`. Sent by the world on entering.
        Region = 85, "REGION", -2;
        Unnamed86 = 86, "aClass375_4437", -2;
        CreateAccountReply = 87, "CREATE_ACCOUNT_REPLY", 1;
        ObjectPrefetch = 88, "OBJECT_PREFETCH", 5;
        CamReset = 89, "CAM_RESET", 0;
        UpdateUid192 = 90, "UPDATE_UID192", 28;
        UpdateSitesettingsCookie = 91, "UPDATE_SITESETTINGS_COOKIE", -1;
        SendPrivateMessage = 92, "SEND_PRIVATE_MESSAGE", -2;
        Unnamed93 = 93, "aClass375_3828", 11;
        VarclanDisable = 94, "VARCLAN_DISABLE", 0;
        MapProjanimHalfsq = 95, "MAP_PROJANIM_HALFSQ", 17;
        OutdatedInterfacePulseEvent = 96, "OUTDATED_INTERFACE_PULSE_EVENT", 8;
        UpdateIgnoreList = 97, "UPDATE_IGNORE_LIST", -2;
        ReflectionCheck = 98, "REFLECTION_CHECK", -2;
        RunCs2Script = 99, "RUN_CS2_SCRIPT", -2;
        ClearVarps = 100, "CLEAR_VARPS", 0;
        FriendlistLoaded = 101, "FRIENDLIST_LOADED", 0;
        UpdateRebootTimer = 102, "UPDATE_REBOOT_TIMER", 2;
        /// The world list, or the player counts of the list the client already has. Layout, encoder
        /// and decoder: `crate::protocol::world_list`. Sent by the lobby in answer to
        /// `REQUEST_WORLD_LIST`.
        WorldList = 103, "WORLD_LIST", -2;
        VorbisSpeechSound = 104, "VORBIS_SPEECH_SOUND", 6;
        IfSetcolor = 105, "IF_SETCOLOR", 6;
        Unnamed106 = 106, "aClass375_4457", 2;
        GroundItemReveal = 107, "GROUND_ITEM_REVEAL", 7;
        VarbitLarge = 108, "VARBIT_LARGE", 6;
        IfSetnpchead = 109, "IF_SETNPCHEAD", 8;
        Cutscene = 110, "CUTSCENE", -2;
        PlayerOption = 111, "PLAYER_OPTION", -1;
        IfSetitem = 112, "IF_SETITEM", 10;
        ObjAnim = 113, "OBJ_ANIM", 6;
        TileMessage = 114, "TILE_MESSAGE", -1;
        /// A player variable whose value fits a signed byte: the value (`readByte`), then the id
        /// (`readUnsignedShortLE128`: low byte + 128, then the high byte). Encoder:
        /// `server_packets::varp`. Sent by the lobby and the world.
        VarpSmall = 115, "VARP_SMALL", 3;
        /// A client variable whose value fits a signed byte: the value (`read128Byte`: sent as 128 −
        /// value), then the id (`readShortLE`). Encoder: `server_packets::varc`. Sent by the world
        /// (variable 1413, before the game's window).
        ClientSetvarcSmall = 116, "CLIENT_SETVARC_SMALL", 3;
        CreateObject = 117, "CREATE_OBJECT", 6;
        CamForceangle = 118, "CAM_FORCEANGLE", 4;
        ClientSetvarcstrLarge = 119, "CLIENT_SETVARCSTR_LARGE", -2;
        ResetSounds = 120, "RESET_SOUNDS", 0;
        IfSettargetparam = 121, "IF_SETTARGETPARAM", 12;
        ResetAllAnimations = 122, "RESET_ALL_ANIMATIONS", 0;
        VarclanSetByte = 123, "VARCLAN_SET_BYTE", 3;
        IfSettext = 124, "IF_SETTEXT", -2;
        RemoveGroundItem = 125, "REMOVE_GROUND_ITEM", 3;
        SpotAnimSpecific = 126, "SPOT_ANIM_SPECIFIC", 12;
        FriendsChatChannel = 127, "FRIENDS_CHAT_CHANNEL", -2;
        MidiSongLocation = 128, "MIDI_SONG_LOCATION", 6;
        SetTarget = 129, "SET_TARGET", 2;
        IfSetplayerheadIgnoreworn = 130, "IF_SETPLAYERHEAD_IGNOREWORN", 10;
        MessageQuickchatClanchannel = 131, "MESSAGE_QUICKCHAT_CLANCHANNEL", -1;
        IfSetretex = 132, "IF_SETRETEX", 9;
        MessagePlayerGroup = 133, "MESSAGE_PLAYER_GROUP", -1;
        Unnamed134 = 134, "aClass375_4453", 2;
        IfSetmodel = 135, "IF_SETMODEL", 8;
        SoundSynth = 136, "SOUND_SYNTH", 8;
        ClansettingsFull = 137, "CLANSETTINGS_FULL", -2;
        AddIgnore = 138, "ADD_IGNORE", -1;
        SpotAnim = 139, "SPOT_ANIM", 8;
        /// One skill: its experience (`readInt`), its id and its level (`readUnsignedByte` each).
        /// Encoder: `server_packets::update_stat`. Sent by the world on entering, once per skill.
        UpdateStat = 140, "UPDATE_STAT", 6;
        VarclanSetInt = 141, "VARCLAN_SET_INT", 6;
        MessageQuickchatFriendsChat = 142, "MESSAGE_QUICKCHAT_FRIENDS_CHAT", -1;
        MapProjanim = 143, "MAP_PROJANIM", 16;
        IfSetrecol = 144, "IF_SETRECOL", 9;
        AnimateNpc = 145, "ANIMATE_NPC", 19;
        Unnamed146 = 146, "aClass375_4497", 0;
        IdentifyHostName = 147, "IDENTIFY_HOST_NAME", 4;
        IfResettargetparam = 148, "IF_RESETTARGETPARAM", 10;
        HintTrail = 149, "HINT_TRAIL", -2;
        IfSetclickmask = 150, "IF_SETCLICKMASK", 5;
        SetDrawOrder = 151, "SET_DRAW_ORDER", 1;
        MessagePublic = 152, "MESSAGE_PUBLIC", -1;
        SetCursor = 153, "SET_CURSOR", -1;
        BlockMinimapState = 154, "BLOCK_MINIMAP_STATE", 1;
        IfOpensubActiveGrounditem = 155, "IF_OPENSUB_ACTIVE_GROUNDITEM", 29;
        Unnamed156 = 156, "aClass375_4507", -2;
        UpdateDob = 157, "UPDATE_DOB", 4;
        IfSetplayerheadOther = 158, "IF_SETPLAYERHEAD_OTHER", 10;
        RequestFps = 159, "REQUEST_FPS", 8;
        /// A line for the chat box (`PacketDecoder`): the type (`readSmart`; 0 = a game message, 99 and
        /// 98 go to the developer console), an effects int, a flags byte; with flag 1 the sender's name
        /// (a string) and with flag 2 its formatted form too; then the message (a string). Strings are
        /// Windows-1252 bytes up to a 0. Encoder: `server_packets::game_message` (no sender). Sent by
        /// the world on entering (the welcome).
        GameMessage = 160, "GAME_MESSAGE", -1;
        DiscordRichPresenceUpdate = 161, "DISCORD_RICH_PRESENCE_UPDATE", -1;
    }
}

/// A packet ready to send: which one, and its payload.
pub type Packet = (ServerProt, Vec<u8>);

/// A packet on the wire: the id as an ISAAC-scrambled smart (`readEncryptedSmart`), then its
/// length when its size is a u8 or a u16, then the payload.
pub fn frame(packet: ServerProt, payload: &[u8], cipher: &mut Isaac) -> Vec<u8> {
    let id = packet.id();
    let mut out = Vec::with_capacity(payload.len() + 4);
    if id < 128 {
        out.push((id as u8).wrapping_add(cipher.next() as u8));
    } else {
        out.push(((id >> 8) as u8 + 128).wrapping_add(cipher.next() as u8));
        out.push((id as u8).wrapping_add(cipher.next() as u8));
    }
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
    fn sizes_match_the_reference() {
        assert_eq!(ServerProt::VarpSmall.size(), Size::Fixed(3));
        assert_eq!(ServerProt::VarpLarge.size(), Size::Fixed(6));
        assert_eq!((ServerProt::Region.id(), ServerProt::Region.size()), (85, Size::Short));
        assert_eq!(ServerProt::from_id(160), Some(ServerProt::GameMessage));
    }

    /// Ids from 128 up take two bytes, the first with 128 added; each byte uses one cipher value.
    #[test]
    fn a_frame_has_a_smart_id_then_the_length() {
        let (_, mut cipher) = Isaac::pair([1, 2, 3, 4]);
        let (_, mut same) = Isaac::pair([1, 2, 3, 4]);
        let keys = [same.next() as u8, same.next() as u8, same.next() as u8];
        let small = frame(ServerProt::RunEnergy, &[100], &mut cipher);
        assert_eq!(small, vec![64u8.wrapping_add(keys[0]), 100]);
        let large = frame(ServerProt::GameMessage, &[7, 8], &mut cipher);
        assert_eq!(large, vec![128u8.wrapping_add(keys[1]), 160u8.wrapping_add(keys[2]), 2, 7, 8]);
    }
}
