//! Friends and ignore lists as the scripts read them (`client.FRIENDS`, `IGNORED_PLAYERS`,
//! `anInt7434`). The server fills them with its friends-list packets; until then the list
//! is "not loaded" and empty.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Friend {
    pub display_name: String,
    /// `username`: the previous name, "" if none.
    pub username: String,
    /// `worldId` (0 offline) and `worldName`.
    pub world: i32,
    pub world_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ignored {
    pub display_name: String,
    /// `lastDisplayName`, "" if none.
    pub last_display_name: String,
}

#[derive(Clone, Debug, Default)]
pub struct Social {
    /// `anInt7434`: 0 not loaded, 1 loading, 2 loaded.
    pub status: i32,
    pub friends: Vec<Friend>,
    pub ignored: Vec<Ignored>,
}

/// `Class76.getCrown(0)` / `(1)`: names shown with a crown start with this 7-character tag.
fn without_crown(name: &str) -> &str {
    if name.starts_with("<img=0>") || name.starts_with("<img=1>") {
        &name[7..]
    } else {
        name
    }
}

impl Social {
    /// `FRIEND_COUNT` (`method4773`): -2 not loaded, -1 loading, else the count.
    pub fn friend_count(&self) -> i32 {
        match self.status {
            0 => -2,
            1 => -1,
            _ => self.friends.len() as i32,
        }
    }

    /// Friend `n` once the list is loaded.
    pub fn friend(&self, n: i32) -> Option<&Friend> {
        (self.status == 2).then(|| usize::try_from(n).ok().and_then(|n| self.friends.get(n))).flatten()
    }

    /// `IGNORE_COUNT` (`method4829`): -1 not loaded, else the count.
    pub fn ignore_count(&self) -> i32 {
        if self.status == 0 {
            -1
        } else {
            self.ignored.len() as i32
        }
    }

    /// Ignored player `n` once the list is loading or loaded.
    pub fn ignored(&self, n: i32) -> Option<&Ignored> {
        (self.status != 0).then(|| usize::try_from(n).ok().and_then(|n| self.ignored.get(n))).flatten()
    }

    /// `Static.isFriend` (after `FRIEND_TEST` drops a crown): a friend's display name, or
    /// this player's own, without case.
    pub fn is_friend(&self, name: &str, me: &str) -> bool {
        let name = without_crown(name);
        self.friends.iter().any(|f| f.display_name.eq_ignore_ascii_case(name)) || name.eq_ignore_ascii_case(me)
    }

    /// `Class280.isIgnored`: the display name or the last one, without case.
    pub fn is_ignored(&self, name: &str) -> bool {
        let name = without_crown(name);
        self.ignored.iter().any(|i| i.display_name.eq_ignore_ascii_case(name) || i.last_display_name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_follow_the_load_state() {
        let mut s = Social::default();
        assert_eq!((s.friend_count(), s.ignore_count()), (-2, -1));
        s.status = 2;
        s.friends.push(Friend { display_name: "Bob".into(), ..Default::default() });
        assert_eq!(s.friend_count(), 1);
        assert!(s.is_friend("<img=1>bob", "me") && s.is_friend("ME", "me") && !s.is_friend("eve", "me"));
    }
}
