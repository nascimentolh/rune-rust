//! Map keys (XTEA) in the OpenRS2 archive `keys.json` format:
//! `[{"archive": 5, "group": 1, "name": "l45_48", "mapsquare": 11568, "key": [a, b, c, d]}, ...]`

use super::error::{CacheError, Result};
use super::xtea::Key;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Clone, Debug)]
pub struct KeyEntry {
    pub archive: u8,
    pub group: u32,
    pub key: Key,
}

/// Other fields in the file (name, mapsquare, name_hash) aren't needed yet and are skipped.
#[derive(Default)]
pub struct MapKeys {
    pub entries: Vec<KeyEntry>,
}

impl MapKeys {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|_| CacheError::Missing(format!("keys file {}", path.display())))?;
        Self::parse(&text)
    }

    pub fn parse(json: &str) -> Result<Self> {
        let entries = serde_json::from_str(json).map_err(|e| CacheError::Corrupt(format!("keys.json: {e}")))?;
        Ok(MapKeys { entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openrs2_format() {
        let keys =
            MapKeys::parse(r#"[{"archive":5,"group":1,"name_hash":-1,"name":"l45_48","mapsquare":11568,"key":[1,-2,3,4]}]"#).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!((keys.entries[0].archive, keys.entries[0].group, keys.entries[0].key), (5, 1, [1, -2, 3, 4]));
    }
}
