//! Client packets that move the player.

use crate::protocol::client_packets;
use crate::server::world::player::Player;

/// `WALK`: the destination is kept for the movement stage, as the reference server's
/// `WalkHandler` keeps `walkRequest` (the stage routes it). Only on the plane they stand on.
pub fn walk(player: &mut Player, payload: &[u8]) {
    if let Some(destination) = client_packets::walk_destination(payload) {
        player.walk_request = Some(destination);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_keeps_the_destination() {
        let mut player = Player::new(1, "bob".into(), Some(1));
        walk(&mut player, &client_packets::walk(false, 3225, 3218).1);
        assert_eq!(player.walk_request, Some((3225, 3218)));
        assert!(player.take_queued().is_empty());
    }
}
