# Game protocol (build 727)

Written by `cargo run -- --protocol-doc` from `src/protocol/server_prot.rs` and
`src/protocol/client_prot.rs`. Don't edit it: change the tables' rows and doc comments and
run the command again (`cargo test` fails while this file is out of date).

Ids, names and sizes are the reference's (`ServerProt.java`, `ClientProt.java`). A packet
is ported when its row has a doc comment saying what its payload is and who uses it; how
to port one is in `docs/SERVER.md`.

On the wire, after login: a server packet's id is a smart (one byte below 128, else two)
with each byte added to the server's ISAAC cipher; a client packet's id is one byte added
to the client's cipher. A "u8 length" or "u16 length" size means that length comes next;
then the payload.

## Server → client

`ServerProt`: 162 packets, 13 ported.

| Id | Name | In code | Size | Ported |
|---:|---|---|---|---|
| 0 | `IF_SETPLAYERHEAD` | `ServerProt::IfSetplayerhead` | 4 |  |
| 1 | `CREATE_CHECK_EMAIL_REPLY` | `ServerProt::CreateCheckEmailReply` | 1 |  |
| 2 | `PROCESS_DEV_CONSOLE_COMMAND` | `ServerProt::ProcessDevConsoleCommand` | u8 length |  |
| 3 | `GROUND_ITEM_COUNT` | `ServerProt::GroundItemCount` | 7 |  |
| 4 | `UPDATE_INV_PARTIAL` | `ServerProt::UpdateInvPartial` | u16 length |  |
| 5 | `UPDATE_INV_FULL` | `ServerProt::UpdateInvFull` | u16 length |  |
| 6 | `NPC_UPDATE` | `ServerProt::NpcUpdate` | u16 length |  |
| 7 | `IF_SETTEXTFONT` | `ServerProt::IfSettextfont` | 8 |  |
| 8 | `VARP_LARGE` | `ServerProt::VarpLarge` | 6 | yes |
| 9 | `LOGOUT_LOBBY` | `ServerProt::LogoutLobby` | 0 | yes |
| 10 | `MESSAGE_PRIVATE_ECHO` | `ServerProt::MessagePrivateEcho` | u16 length |  |
| 11 | `aClass375_4362` | `ServerProt::Unnamed11` | 2 |  |
| 12 | `CLIENT_SETVARC_LARGE` | `ServerProt::ClientSetvarcLarge` | 6 | yes |
| 13 | `IF_MOVESUB` | `ServerProt::IfMovesub` | 8 |  |
| 14 | `PLAYER_WEIGHT` | `ServerProt::PlayerWeight` | 2 |  |
| 15 | `UPDATE_ZONE_PARTIAL_FOLLOWS` | `ServerProt::UpdateZonePartialFollows` | 3 |  |
| 16 | `OPEN_URL` | `ServerProt::OpenUrl` | u16 length |  |
| 17 | `aClass375_4368` | `ServerProt::Unnamed17` | 3 |  |
| 18 | `VORBIS_SOUND` | `ServerProt::VorbisSound` | 8 |  |
| 19 | `UPDATE_FRIENDCHAT_CHANNEL_SINGLEUSER` | `ServerProt::UpdateFriendchatChannelSingleuser` | u8 length |  |
| 20 | `OUTDATED_SET_THEORA_STRING_SOMETHING` | `ServerProt::OutdatedSetTheoraStringSomething` | 6 |  |
| 21 | `SHOW_FACE_HERE` | `ServerProt::ShowFaceHere` | 1 |  |
| 22 | `MESSAGE_QUICKCHAT_PRIVATE` | `ServerProt::MessageQuickchatPrivate` | u8 length |  |
| 23 | `IF_OPENSUB_ACTIVE_OBJECT` | `ServerProt::IfOpensubActiveObject` | 32 |  |
| 24 | `CAM_LOOKAT` | `ServerProt::CamLookat` | 6 |  |
| 25 | `MESSAGE_FRIENDS_CHAT` | `ServerProt::MessageFriendsChat` | u8 length |  |
| 26 | `VARCLAN_SET_LONG` | `ServerProt::VarclanSetLong` | 10 |  |
| 27 | `CREATE_GROUND_ITEM` | `ServerProt::CreateGroundItem` | 5 |  |
| 28 | `CLANCHANNEL_DELTA` | `ServerProt::ClanchannelDelta` | u16 length |  |
| 29 | `PING` | `ServerProt::Ping` | 0 |  |
| 30 | `CHAT_FILTER_SETTINGS` | `ServerProt::ChatFilterSettings` | 2 |  |
| 31 | `MESSAGE_QUICKCHAT_PRIVATE_ECHO` | `ServerProt::MessageQuickchatPrivateEcho` | u8 length |  |
| 32 | `LOYALTY_UPDATE` | `ServerProt::LoyaltyUpdate` | 5 |  |
| 33 | `IF_SETHIDE` | `ServerProt::IfSethide` | 5 |  |
| 34 | `IF_OPENSUB_ACTIVE_PLAYER` | `ServerProt::IfOpensubActivePlayer` | 25 |  |
| 35 | `IF_SETGRAPHIC` | `ServerProt::IfSetgraphic` | 8 |  |
| 36 | `IF_SETTEXTANTIMACRO` | `ServerProt::IfSettextantimacro` | 5 |  |
| 37 | `IF_OPENTOP` | `ServerProt::IfOpentop` | 19 | yes |
| 38 | `IF_OPENSUB` | `ServerProt::IfOpensub` | 23 | yes |
| 39 | `CAM_MOVETO` | `ServerProt::CamMoveto` | 6 |  |
| 40 | `aClass375_4399` | `ServerProt::Unnamed40` | 4 |  |
| 41 | `UPDATE_ZONE_FULL_FOLLOWS` | `ServerProt::UpdateZoneFullFollows` | 3 |  |
| 42 | `UPDATE_INV_STOP_TRANSMIT` | `ServerProt::UpdateInvStopTransmit` | 3 |  |
| 43 | `PLAYER_UPDATE` | `ServerProt::PlayerUpdate` | u16 length |  |
| 44 | `IF_SETANGLE` | `ServerProt::IfSetangle` | 10 |  |
| 45 | `VARCLAN_ENABLE` | `ServerProt::VarclanEnable` | 0 |  |
| 46 | `CLANSETTINGS_DELTA` | `ServerProt::ClansettingsDelta` | u16 length |  |
| 47 | `NPC_UPDATE_LARGE` | `ServerProt::NpcUpdateLarge` | u16 length |  |
| 48 | `IF_SETPOSITION` | `ServerProt::IfSetposition` | 8 |  |
| 49 | `CLANCHANNEL_FULL` | `ServerProt::ClanchannelFull` | u16 length |  |
| 50 | `SET_CLAN_STRING` | `ServerProt::SetClanString` | u8 length |  |
| 51 | `DYNAMIC_MAP_REGION` | `ServerProt::DynamicMapRegion` | u16 length |  |
| 52 | `DEPRECATED_52_CLIENTPACKET_97` | `ServerProt::Deprecated52Clientpacket97` | 1 |  |
| 53 | `IF_SETPLAYERMODEL_OTHER` | `ServerProt::IfSetplayermodelOther` | 10 |  |
| 54 | `CLIENT_SETVARCSTR_SMALL` | `ServerProt::ClientSetvarcstrSmall` | u8 length |  |
| 55 | `IF_SETSCROLLPOS` | `ServerProt::IfSetscrollpos` | 6 |  |
| 56 | `PROJANIM_SPECIFIC` | `ServerProt::ProjanimSpecific` | 22 |  |
| 57 | `UPDATE_GE_SLOT` | `ServerProt::UpdateGeSlot` | 20 |  |
| 58 | `QUICK_HOP_WORLDS` | `ServerProt::QuickHopWorlds` | u8 length |  |
| 59 | `IF_OPENSUB_ACTIVE_NPC` | `ServerProt::IfOpensubActiveNpc` | 25 |  |
| 60 | `MUSIC_EFFECT` | `ServerProt::MusicEffect` | 6 |  |
| 61 | `REDUCE_ATTACK_PRIORITY` | `ServerProt::ReduceAttackPriority` | 1 |  |
| 62 | `LOGOUT_FULL` | `ServerProt::LogoutFull` | 0 | yes |
| 63 | `MUSIC_TRACK` | `ServerProt::MusicTrack` | 4 |  |
| 64 | `RUN_ENERGY` | `ServerProt::RunEnergy` | 1 | yes |
| 65 | `UPDATE_ZONE_PARTIAL_ENCLOSED` | `ServerProt::UpdateZonePartialEnclosed` | u16 length |  |
| 66 | `CAM_SMOOTHRESET` | `ServerProt::CamSmoothreset` | 0 |  |
| 67 | `CHAT_FILTER_SETTINGS_PRIVATECHAT` | `ServerProt::ChatFilterSettingsPrivatechat` | 1 |  |
| 68 | `VARBIT_SMALL` | `ServerProt::VarbitSmall` | 3 |  |
| 69 | `DESTROY_OBJECT` | `ServerProt::DestroyObject` | 2 |  |
| 70 | `MINIMAP_FLAG` | `ServerProt::MinimapFlag` | 2 |  |
| 71 | `CAM_SHAKE` | `ServerProt::CamShake` | 6 |  |
| 72 | `IF_SETPLAYERMODEL` | `ServerProt::IfSetplayermodel` | 4 |  |
| 73 | `DEBUG_SERVER_TRIGGERS` | `ServerProt::DebugServerTriggers` | u8 length |  |
| 74 | `FRIEND_STATUS` | `ServerProt::FriendStatus` | u16 length |  |
| 75 | `JCOINS_UPDATE` | `ServerProt::JcoinsUpdate` | 4 |  |
| 76 | `APPLY_DEBUG` | `ServerProt::ApplyDebug` | 2 |  |
| 77 | `PRELOAD_SONG` | `ServerProt::PreloadSong` | 2 |  |
| 78 | `IF_CLOSESUB` | `ServerProt::IfClosesub` | 4 |  |
| 79 | `HINT_ARROW` | `ServerProt::HintArrow` | 14 |  |
| 80 | `OBJ_ANIM_SPECIFIC` | `ServerProt::ObjAnimSpecific` | 9 |  |
| 81 | `MESSAGE_CLANCHANNEL` | `ServerProt::MessageClanchannel` | u8 length |  |
| 82 | `IF_SETANIM` | `ServerProt::IfSetanim` | 8 |  |
| 83 | `MESSAGE_QUICKCHAT_PLAYER_GROUP` | `ServerProt::MessageQuickchatPlayerGroup` | u8 length |  |
| 84 | `CUSTOMIZE_OBJECT` | `ServerProt::CustomizeObject` | u8 length |  |
| 85 | `REGION` | `ServerProt::Region` | u16 length | yes |
| 86 | `aClass375_4437` | `ServerProt::Unnamed86` | u16 length |  |
| 87 | `CREATE_ACCOUNT_REPLY` | `ServerProt::CreateAccountReply` | 1 |  |
| 88 | `OBJECT_PREFETCH` | `ServerProt::ObjectPrefetch` | 5 |  |
| 89 | `CAM_RESET` | `ServerProt::CamReset` | 0 |  |
| 90 | `UPDATE_UID192` | `ServerProt::UpdateUid192` | 28 |  |
| 91 | `UPDATE_SITESETTINGS_COOKIE` | `ServerProt::UpdateSitesettingsCookie` | u8 length |  |
| 92 | `SEND_PRIVATE_MESSAGE` | `ServerProt::SendPrivateMessage` | u16 length |  |
| 93 | `aClass375_3828` | `ServerProt::Unnamed93` | 11 |  |
| 94 | `VARCLAN_DISABLE` | `ServerProt::VarclanDisable` | 0 |  |
| 95 | `MAP_PROJANIM_HALFSQ` | `ServerProt::MapProjanimHalfsq` | 17 |  |
| 96 | `OUTDATED_INTERFACE_PULSE_EVENT` | `ServerProt::OutdatedInterfacePulseEvent` | 8 |  |
| 97 | `UPDATE_IGNORE_LIST` | `ServerProt::UpdateIgnoreList` | u16 length |  |
| 98 | `REFLECTION_CHECK` | `ServerProt::ReflectionCheck` | u16 length |  |
| 99 | `RUN_CS2_SCRIPT` | `ServerProt::RunCs2Script` | u16 length |  |
| 100 | `CLEAR_VARPS` | `ServerProt::ClearVarps` | 0 |  |
| 101 | `FRIENDLIST_LOADED` | `ServerProt::FriendlistLoaded` | 0 |  |
| 102 | `UPDATE_REBOOT_TIMER` | `ServerProt::UpdateRebootTimer` | 2 |  |
| 103 | `WORLD_LIST` | `ServerProt::WorldList` | u16 length | yes |
| 104 | `VORBIS_SPEECH_SOUND` | `ServerProt::VorbisSpeechSound` | 6 |  |
| 105 | `IF_SETCOLOR` | `ServerProt::IfSetcolor` | 6 |  |
| 106 | `aClass375_4457` | `ServerProt::Unnamed106` | 2 |  |
| 107 | `GROUND_ITEM_REVEAL` | `ServerProt::GroundItemReveal` | 7 |  |
| 108 | `VARBIT_LARGE` | `ServerProt::VarbitLarge` | 6 |  |
| 109 | `IF_SETNPCHEAD` | `ServerProt::IfSetnpchead` | 8 |  |
| 110 | `CUTSCENE` | `ServerProt::Cutscene` | u16 length |  |
| 111 | `PLAYER_OPTION` | `ServerProt::PlayerOption` | u8 length |  |
| 112 | `IF_SETITEM` | `ServerProt::IfSetitem` | 10 |  |
| 113 | `OBJ_ANIM` | `ServerProt::ObjAnim` | 6 |  |
| 114 | `TILE_MESSAGE` | `ServerProt::TileMessage` | u8 length |  |
| 115 | `VARP_SMALL` | `ServerProt::VarpSmall` | 3 | yes |
| 116 | `CLIENT_SETVARC_SMALL` | `ServerProt::ClientSetvarcSmall` | 3 | yes |
| 117 | `CREATE_OBJECT` | `ServerProt::CreateObject` | 6 |  |
| 118 | `CAM_FORCEANGLE` | `ServerProt::CamForceangle` | 4 |  |
| 119 | `CLIENT_SETVARCSTR_LARGE` | `ServerProt::ClientSetvarcstrLarge` | u16 length |  |
| 120 | `RESET_SOUNDS` | `ServerProt::ResetSounds` | 0 |  |
| 121 | `IF_SETTARGETPARAM` | `ServerProt::IfSettargetparam` | 12 |  |
| 122 | `RESET_ALL_ANIMATIONS` | `ServerProt::ResetAllAnimations` | 0 |  |
| 123 | `VARCLAN_SET_BYTE` | `ServerProt::VarclanSetByte` | 3 |  |
| 124 | `IF_SETTEXT` | `ServerProt::IfSettext` | u16 length |  |
| 125 | `REMOVE_GROUND_ITEM` | `ServerProt::RemoveGroundItem` | 3 |  |
| 126 | `SPOT_ANIM_SPECIFIC` | `ServerProt::SpotAnimSpecific` | 12 |  |
| 127 | `FRIENDS_CHAT_CHANNEL` | `ServerProt::FriendsChatChannel` | u16 length |  |
| 128 | `MIDI_SONG_LOCATION` | `ServerProt::MidiSongLocation` | 6 |  |
| 129 | `SET_TARGET` | `ServerProt::SetTarget` | 2 |  |
| 130 | `IF_SETPLAYERHEAD_IGNOREWORN` | `ServerProt::IfSetplayerheadIgnoreworn` | 10 |  |
| 131 | `MESSAGE_QUICKCHAT_CLANCHANNEL` | `ServerProt::MessageQuickchatClanchannel` | u8 length |  |
| 132 | `IF_SETRETEX` | `ServerProt::IfSetretex` | 9 |  |
| 133 | `MESSAGE_PLAYER_GROUP` | `ServerProt::MessagePlayerGroup` | u8 length |  |
| 134 | `aClass375_4453` | `ServerProt::Unnamed134` | 2 |  |
| 135 | `IF_SETMODEL` | `ServerProt::IfSetmodel` | 8 |  |
| 136 | `SOUND_SYNTH` | `ServerProt::SoundSynth` | 8 |  |
| 137 | `CLANSETTINGS_FULL` | `ServerProt::ClansettingsFull` | u16 length |  |
| 138 | `ADD_IGNORE` | `ServerProt::AddIgnore` | u8 length |  |
| 139 | `SPOT_ANIM` | `ServerProt::SpotAnim` | 8 |  |
| 140 | `UPDATE_STAT` | `ServerProt::UpdateStat` | 6 | yes |
| 141 | `VARCLAN_SET_INT` | `ServerProt::VarclanSetInt` | 6 |  |
| 142 | `MESSAGE_QUICKCHAT_FRIENDS_CHAT` | `ServerProt::MessageQuickchatFriendsChat` | u8 length |  |
| 143 | `MAP_PROJANIM` | `ServerProt::MapProjanim` | 16 |  |
| 144 | `IF_SETRECOL` | `ServerProt::IfSetrecol` | 9 |  |
| 145 | `ANIMATE_NPC` | `ServerProt::AnimateNpc` | 19 |  |
| 146 | `aClass375_4497` | `ServerProt::Unnamed146` | 0 |  |
| 147 | `IDENTIFY_HOST_NAME` | `ServerProt::IdentifyHostName` | 4 |  |
| 148 | `IF_RESETTARGETPARAM` | `ServerProt::IfResettargetparam` | 10 |  |
| 149 | `HINT_TRAIL` | `ServerProt::HintTrail` | u16 length |  |
| 150 | `IF_SETCLICKMASK` | `ServerProt::IfSetclickmask` | 5 |  |
| 151 | `SET_DRAW_ORDER` | `ServerProt::SetDrawOrder` | 1 |  |
| 152 | `MESSAGE_PUBLIC` | `ServerProt::MessagePublic` | u8 length |  |
| 153 | `SET_CURSOR` | `ServerProt::SetCursor` | u8 length |  |
| 154 | `BLOCK_MINIMAP_STATE` | `ServerProt::BlockMinimapState` | 1 |  |
| 155 | `IF_OPENSUB_ACTIVE_GROUNDITEM` | `ServerProt::IfOpensubActiveGrounditem` | 29 |  |
| 156 | `aClass375_4507` | `ServerProt::Unnamed156` | u16 length |  |
| 157 | `UPDATE_DOB` | `ServerProt::UpdateDob` | 4 |  |
| 158 | `IF_SETPLAYERHEAD_OTHER` | `ServerProt::IfSetplayerheadOther` | 10 |  |
| 159 | `REQUEST_FPS` | `ServerProt::RequestFps` | 8 |  |
| 160 | `GAME_MESSAGE` | `ServerProt::GameMessage` | u8 length | yes |
| 161 | `DISCORD_RICH_PRESENCE_UPDATE` | `ServerProt::DiscordRichPresenceUpdate` | u8 length |  |

### Ported: server → client

**8 `VARP_LARGE`** (6)

A player variable whose value doesn't fit a signed byte: the value (`readIntV2`: bytes 2, 1,
4, 3 of the big-endian int), then the id (`readUnsignedShortLE128`: low byte + 128, then
the high byte). Encoder: `server_packets::varp`. Sent by the lobby and the world.

**9 `LOGOUT_LOBBY`** (0)

No body. `CursorIndexLoader.logout(Class9.aBool71)`: back to the lobby when the player
came through it, else to the login screen. Encoder: `server_packets::logout`. Sent by the
world for the logout tab's "Exit to Lobby".

**12 `CLIENT_SETVARC_LARGE`** (6)

A client variable whose value doesn't fit a signed byte: the id (`readShortLE`), then the
value (`readIntV2`: bytes 2, 1, 4, 3). Encoder: `server_packets::varc`.

**37 `IF_OPENTOP`** (19)

Opens the top-level window (`PacketDecoder`): a key int (V1), a flags int (V1), the window
id (u16, low byte + 128), the type (128 − byte), two ints. This server sends the keys, the
type and the ints as 0. Encoder: `server_packets::open_top`. Sent by the world on entering
and when the window mode changes.

**38 `IF_OPENSUB`** (23)

Opens an interface in a component (`PacketDecoder`, `IdentiKitIndexLoader.openSub`): a key
int, a flags int (V2), an int (V2), the interface (u16 LE, low byte + 128), an int (LE), the
parent component (int: interface << 16 | component), the type (`readUnsignedByteC`: sent
negated; 1 = an overlay). This server sends the keys and ints as 0. Encoder:
`server_packets::open_sub`. Sent by the world for the side panels, the orbs and the chat.

**62 `LOGOUT_FULL`** (0)

No body. `CursorIndexLoader.logout(false)`: to the login screen. Encoder:
`server_packets::logout`. Sent by the world for the logout tab's "Exit to Login".

**64 `RUN_ENERGY`** (1)

The run energy, one unsigned byte (0–100): `client.RUN_ENERGY`. Encoder:
`server_packets::run_energy`. Sent by the world on entering.

**85 `REGION`** (u16 length)

The first game packet of a world login (`Login.java`, stages 167 → 173 → 156): every
player's position in bits (`Class197.method3203`), then the map scene
(`MapRegion.loadMapSceneNormal`) with the XTEA keys of the map regions around. Layout and
encoder: `crate::protocol::region`. Sent by the world on entering.

**103 `WORLD_LIST`** (u16 length)

The world list, or the player counts of the list the client already has. Layout, encoder
and decoder: `crate::protocol::world_list`. Sent by the lobby in answer to
`REQUEST_WORLD_LIST`.

**115 `VARP_SMALL`** (3)

A player variable whose value fits a signed byte: the value (`readByte`), then the id
(`readUnsignedShortLE128`: low byte + 128, then the high byte). Encoder:
`server_packets::varp`. Sent by the lobby and the world.

**116 `CLIENT_SETVARC_SMALL`** (3)

A client variable whose value fits a signed byte: the value (`read128Byte`: sent as 128 −
value), then the id (`readShortLE`). Encoder: `server_packets::varc`. Sent by the world
(variable 1413, before the game's window).

**140 `UPDATE_STAT`** (6)

One skill: its experience (`readInt`), its id and its level (`readUnsignedByte` each).
Encoder: `server_packets::update_stat`. Sent by the world on entering, once per skill.

**160 `GAME_MESSAGE`** (u8 length)

A line for the chat box (`PacketDecoder`): the type (`readSmart`; 0 = a game message, 99 and
98 go to the developer console), an effects int, a flags byte; with flag 1 the sender's name
(a string) and with flag 2 its formatted form too; then the message (a string). Strings are
Windows-1252 bytes up to a 0. Encoder: `server_packets::game_message` (no sender). Sent by
the world on entering (the welcome).

## Client → server

`ClientProt`: 104 packets, 12 ported.

| Id | Name | In code | Size | Ported |
|---:|---|---|---|---|
| 0 | `PING` | `ClientProt::Ping` | 0 |  |
| 1 | `PLAYER_OP6` | `ClientProt::PlayerOp6` | 3 |  |
| 2 | `SOUND_EFFECT_MUSIC_ENDED` | `ClientProt::SoundEffectMusicEnded` | 4 |  |
| 3 | `NPC_EXAMINE` | `ClientProt::NpcExamine` | 3 |  |
| 4 | `IF_ON_IF` | `ClientProt::IfOnIf` | 16 |  |
| 5 | `WORLD_MAP_CLICK` | `ClientProt::WorldMapClick` | 4 |  |
| 6 | `PLAYER_OP2` | `ClientProt::PlayerOp2` | 3 |  |
| 7 | `FC_SET_RANK` | `ClientProt::FcSetRank` | u8 length |  |
| 8 | `GROUND_ITEM_OP4` | `ClientProt::GroundItemOp4` | 7 |  |
| 9 | `IF_OP4` | `ClientProt::IfOp4` | 8 | yes |
| 10 | `SEND_PREFERENCES` | `ClientProt::SendPreferences` | u8 length |  |
| 11 | `RESUME_HSLDIALOG` | `ClientProt::ResumeHsldialog` | 2 |  |
| 12 | `REMOVE_IGNORE` | `ClientProt::RemoveIgnore` | u8 length |  |
| 13 | `IF_ON_PLAYER` | `ClientProt::IfOnPlayer` | 11 |  |
| 14 | `QUICKCHAT_PRIVATE` | `ClientProt::QuickchatPrivate` | u8 length |  |
| 15 | `SEND_PRIVATE_MESSAGE` | `ClientProt::SendPrivateMessage` | u16 length |  |
| 16 | `NPC_OP2` | `ClientProt::NpcOp2` | 3 |  |
| 17 | `GRAND_EXCHANGE_ITEM_SELECT` | `ClientProt::GrandExchangeItemSelect` | 2 |  |
| 18 | `SONG_LOADED` | `ClientProt::SongLoaded` | 4 |  |
| 19 | `IF_OP6` | `ClientProt::IfOp6` | 8 | yes |
| 20 | `CHAT_SETFILTER` | `ClientProt::ChatSetfilter` | 3 |  |
| 21 | `IF_OP8` | `ClientProt::IfOp8` | 8 | yes |
| 22 | `IF_OP9` | `ClientProt::IfOp9` | 8 | yes |
| 23 | `IF_OP7` | `ClientProt::IfOp7` | 8 | yes |
| 24 | `GROUND_ITEM_OP1` | `ClientProt::GroundItemOp1` | 7 |  |
| 25 | `GROUND_ITEM_OP2` | `ClientProt::GroundItemOp2` | 7 |  |
| 26 | `ADD_FRIEND` | `ClientProt::AddFriend` | u8 length |  |
| 27 | `IF_OP2` | `ClientProt::IfOp2` | 8 | yes |
| 28 | `KEY_PRESS` | `ClientProt::KeyPress` | u16 length |  |
| 29 | `REMOVE_FRIEND` | `ClientProt::RemoveFriend` | u8 length |  |
| 30 | `CHAT_TYPE` | `ClientProt::ChatType` | 1 |  |
| 31 | `PLAYER_OP3` | `ClientProt::PlayerOp3` | 3 |  |
| 32 | `OBJECT_OP4` | `ClientProt::ObjectOp4` | 9 |  |
| 33 | `WALK` | `ClientProt::Walk` | 5 |  |
| 34 | `ADD_IGNORE` | `ClientProt::AddIgnore` | u8 length |  |
| 35 | `BUG_REPORT` | `ClientProt::BugReport` | u16 length |  |
| 36 | `REFLECTION_CHECK` | `ClientProt::ReflectionCheck` | u8 length |  |
| 37 | `UNK_37` | `ClientProt::Unk37` | 2 |  |
| 38 | `OBJECT_OP3` | `ClientProt::ObjectOp3` | 9 |  |
| 39 | `UNUSED_CLAN_OP` | `ClientProt::UnusedClanOp` | u8 length |  |
| 40 | `MOVE_MOUSE` | `ClientProt::MoveMouse` | u8 length |  |
| 41 | `IF_ON_NPC` | `ClientProt::IfOnNpc` | 11 |  |
| 42 | `MINI_WALK` | `ClientProt::MiniWalk` | 18 |  |
| 43 | `GROUND_ITEM_OP5` | `ClientProt::GroundItemOp5` | 7 |  |
| 44 | `SEND_FPS` | `ClientProt::SendFps` | 9 |  |
| 45 | `CUTSCENE_FINISHED` | `ClientProt::CutsceneFinished` | 1 |  |
| 46 | `IF_ON_TILE` | `ClientProt::IfOnTile` | 12 |  |
| 47 | `REQUEST_WORLD_LIST` | `ClientProt::RequestWorldList` | 4 | yes |
| 48 | `OBJECT_OP5` | `ClientProt::ObjectOp5` | 9 |  |
| 49 | `IF_CONTINUE` | `ClientProt::IfContinue` | 6 |  |
| 50 | `NPC_OP3` | `ClientProt::NpcOp3` | 3 |  |
| 51 | `PLAYER_OP7` | `ClientProt::PlayerOp7` | 3 |  |
| 52 | `EMAIL_VALIDATION_SUBMIT_CODE` | `ClientProt::EmailValidationSubmitCode` | u8 length |  |
| 53 | `PLAYER_OP9` | `ClientProt::PlayerOp9` | 3 |  |
| 54 | `GROUND_ITEM_OP3` | `ClientProt::GroundItemOp3` | 7 |  |
| 55 | `TRANSMITVAR_VERIFYID` | `ClientProt::TransmitvarVerifyid` | 4 |  |
| 56 | `LOBBY_HYPERLINK` | `ClientProt::LobbyHyperlink` | u16 length |  |
| 57 | `MOUSE_BUTTON_CLICK` | `ClientProt::MouseButtonClick` | 7 |  |
| 58 | `RESUME_COUNTDIALOG` | `ClientProt::ResumeCountdialog` | 4 |  |
| 59 | `MOUSE_CLICK` | `ClientProt::MouseClick` | 6 |  |
| 60 | `CLOSE_INTERFACE` | `ClientProt::CloseInterface` | 0 |  |
| 61 | `GROUND_ITEM_EXAMINE` | `ClientProt::GroundItemExamine` | 7 |  |
| 62 | `CLIENT_FOCUS` | `ClientProt::ClientFocus` | 1 |  |
| 63 | `UNK_63` | `ClientProt::Unk63` | 4 |  |
| 64 | `QUICKCHAT_PUBLIC` | `ClientProt::QuickchatPublic` | u8 length |  |
| 65 | `NPC_OP1` | `ClientProt::NpcOp1` | 3 |  |
| 66 | `PLAYER_OP1` | `ClientProt::PlayerOp1` | 3 |  |
| 67 | `IF_ON_GROUND_ITEM` | `ClientProt::IfOnGroundItem` | 15 |  |
| 68 | `IF_OP3` | `ClientProt::IfOp3` | 8 | yes |
| 69 | `RESUME_CLANFORUMQFCDIALOG` | `ClientProt::ResumeClanforumqfcdialog` | u8 length |  |
| 70 | `PLAYER_OP10` | `ClientProt::PlayerOp10` | 3 |  |
| 71 | `FC_JOIN` | `ClientProt::FcJoin` | u8 length |  |
| 72 | `IF_OP5` | `ClientProt::IfOp5` | 8 | yes |
| 73 | `OBJECT_EXAMINE` | `ClientProt::ObjectExamine` | 9 |  |
| 74 | `IF_DRAG_ONTO_IF` | `ClientProt::IfDragOntoIf` | 16 |  |
| 75 | `OBJECT_OP1` | `ClientProt::ObjectOp1` | 9 |  |
| 76 | `REGION_LOADED_CONFIRM` | `ClientProt::RegionLoadedConfirm` | 0 |  |
| 77 | `NPC_OP4` | `ClientProt::NpcOp4` | 3 |  |
| 78 | `MOVE_MOUSE_2` | `ClientProt::MoveMouse2` | u8 length |  |
| 79 | `UNK_79` | `ClientProt::Unk79` | 1 |  |
| 80 | `RESUME_NAMEDIALOG` | `ClientProt::ResumeNamedialog` | u8 length |  |
| 81 | `IF_OP10` | `ClientProt::IfOp10` | 8 | yes |
| 82 | `UNK_82` | `ClientProt::Unk82` | 4 |  |
| 83 | `MOVE_CAMERA` | `ClientProt::MoveCamera` | 4 |  |
| 84 | `SCREEN_SIZE` | `ClientProt::ScreenSize` | 6 | yes |
| 85 | `COMMAND` | `ClientProt::Command` | u8 length |  |
| 86 | `CHAT` | `ClientProt::Chat` | u8 length |  |
| 87 | `RESUME_TEXTDIALOG` | `ClientProt::ResumeTextdialog` | u8 length |  |
| 88 | `WRITE_PING` | `ClientProt::WritePing` | 2 |  |
| 89 | `PLAYER_OP4` | `ClientProt::PlayerOp4` | 3 |  |
| 90 | `CLANCHANNEL_KICKUSER` | `ClientProt::ClanchannelKickuser` | u8 length |  |
| 91 | `FC_KICK` | `ClientProt::FcKick` | u8 length |  |
| 92 | `EMAIL_VALIDATION_ADD_NEW_ADDRESS` | `ClientProt::EmailValidationAddNewAddress` | u16 length |  |
| 93 | `OBJECT_OP2` | `ClientProt::ObjectOp2` | 9 |  |
| 94 | `PLAYER_OP8` | `ClientProt::PlayerOp8` | 3 |  |
| 95 | `NPC_OP5` | `ClientProt::NpcOp5` | 3 |  |
| 96 | `IF_OP1` | `ClientProt::IfOp1` | 8 | yes |
| 97 | `DEPRECATED_97_SERVERPACKET_52` | `ClientProt::Deprecated97Serverpacket52` | u8 length |  |
| 98 | `IF_ON_OBJECT` | `ClientProt::IfOnObject` | 17 |  |
| 99 | `EMAIL_VALIDATION_CHANGE_ADDRESS` | `ClientProt::EmailValidationChangeAddress` | u16 length |  |
| 100 | `REPORT_ABUSE` | `ClientProt::ReportAbuse` | u8 length |  |
| 101 | `SEND_SIGN_UP_FORM` | `ClientProt::SendSignUpForm` | u16 length |  |
| 102 | `CHECK_EMAIL_VALIDITY` | `ClientProt::CheckEmailValidity` | u16 length |  |
| 103 | `PLAYER_OP5` | `ClientProt::PlayerOp5` | 3 |  |

### Ported: client → server

**9 `IF_OP4`** (8)

Option 4 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**19 `IF_OP6`** (8)

Option 6 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**21 `IF_OP8`** (8)

Option 8 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**22 `IF_OP9`** (8)

Option 9 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**23 `IF_OP7`** (8)

Option 7 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**27 `IF_OP2`** (8)

Option 2 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**47 `REQUEST_WORLD_LIST`** (4)

The lobby asks for the world list: the checksum (int) of the list it holds, 0 for none.
Encoder and decoder: `client_packets::request_world_list`, `client_packets::world_list_held`.
Handled by the lobby (`server::lobby`), which answers with `WORLD_LIST`.

**68 `IF_OP3`** (8)

Option 3 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**72 `IF_OP5`** (8)

Option 5 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**81 `IF_OP10`** (8)

Option 10 of a component was clicked: the same body as `IF_OP1`. Sent by the client
(`client_packets::if_op`); the server has no handler for it yet.

**84 `SCREEN_SIZE`** (6)

The window mode or game area changed while in the game (`Class388.method6692`): the mode
(`Class158.windowedMode`: 1 fixed, 2 resizable, 3 full screen), the game area's width and
height (u16 each), the anti-aliasing preference (byte). Encoder and decoder:
`client_packets::screen_size`, `client_packets::screen_size_mode`. Handled by the world
(`server::world::handlers::interface`), which opens the other game window when the mode needs it.

**96 `IF_OP1`** (8)

Option 1 of a component was clicked (`CutsceneAction_Sub10.method14603`,
`Class350_Sub3.method12590`): the component (int LE: interface << 16 | component), its item
(u16, 65535 for none), its slot (u16, low byte + 128). Encoder and decoder:
`client_packets::if_op`, `client_packets::if_op_component`. Handled by the world
(`server::world::handlers::interface`) for the logout tab's two buttons.
