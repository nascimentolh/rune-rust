//! How the client orders text (`Class455.method7557`): string comparisons in scripts and the
//! world list's name columns.

/// `method7557` for unaccented text: case-insensitive order first, then length, then case.
///
/// ponytail: accented letters are compared by code point; the reference expands them
/// (`method7778`) and ranks them (`method13422`).
pub fn compare(a: &str, b: &str) -> i32 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    for (x, y) in a.iter().zip(&b) {
        let (lx, ly) = (x.to_ascii_lowercase(), y.to_ascii_lowercase());
        if lx != ly {
            return lx as i32 - ly as i32;
        }
    }
    if a.len() != b.len() {
        return if a.len() < b.len() { -1 } else { 1 };
    }
    a.iter().zip(&b).find(|(x, y)| x != y).map_or(0, |(x, y)| *x as i32 - *y as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_like_the_client() {
        assert_eq!(compare("abc", "ABC"), 'a' as i32 - 'A' as i32);
        assert_eq!(compare("ab", "abc"), -1);
        assert!(compare("b", "a") > 0);
        assert_eq!(compare("same", "same"), 0);
    }
}
