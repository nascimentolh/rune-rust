//! Client packets about the game's window and its interfaces.

use crate::protocol::client_packets;
use crate::server::world::interfaces;
use crate::server::world::player::Player;

/// `SCREEN_SIZE`: a new window mode opens the other game window, with everything in it, when
/// the mode needs it (this server's choice of windows).
pub fn screen_size(player: &mut Player, payload: &[u8]) {
    let Some(mode) = client_packets::screen_size_mode(payload) else { return };
    let wanted = interfaces::game_window(Some(mode));
    if wanted != player.window {
        player.window = wanted;
        player.send_all(interfaces::open_window(wanted));
    }
}

/// `IF_OP1`: the logout tab's buttons (182:6 and 182:13) send the player to the lobby or to
/// the login screen. No other component's option is ported.
pub fn option_1(player: &mut Player, payload: &[u8]) {
    match client_packets::if_op_component(payload) {
        Some(interfaces::EXIT_TO_LOBBY) => player.log_out(true),
        Some(interfaces::EXIT_TO_LOGIN) => player.log_out(false),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::server_packets;
    use crate::protocol::server_prot::ServerProt;

    /// `SCREEN_SIZE` with another mode opens the other game window, once.
    #[test]
    fn a_new_window_mode_opens_the_other_game_window() {
        let mut player = Player::new(1, "bob".into(), Some(1));
        let (_, resizable) = client_packets::screen_size(2, 1024, 768, 0);
        let (_, fixed) = client_packets::screen_size(1, 765, 503, 0);

        screen_size(&mut player, &resizable);
        let opened = player.take_queued();
        assert_eq!((player.window, opened.len()), (746, 24));
        assert_eq!(opened[0], server_packets::open_top(746));

        screen_size(&mut player, &resizable);
        assert!(player.take_queued().is_empty());

        screen_size(&mut player, &fixed);
        assert_eq!((player.window, player.take_queued()[0].clone()), (548, server_packets::open_top(548)));

        screen_size(&mut player, &[]);
        assert!(player.take_queued().is_empty());
    }

    #[test]
    fn the_logout_buttons_send_the_player_out() {
        for (component, packet) in
            [(interfaces::EXIT_TO_LOBBY, ServerProt::LogoutLobby), (interfaces::EXIT_TO_LOGIN, ServerProt::LogoutFull)]
        {
            let mut player = Player::new(1, "bob".into(), None);
            option_1(&mut player, &client_packets::if_op(1, component as i32, -1, -1).1);
            assert!(player.leaving);
            assert_eq!(player.take_queued(), vec![(packet, Vec::new())]);
        }
        // Any other component does nothing.
        let mut player = Player::new(1, "bob".into(), None);
        option_1(&mut player, &client_packets::if_op(1, 548 << 16 | 1, -1, -1).1);
        assert!(!player.leaving && player.take_queued().is_empty());
    }
}
