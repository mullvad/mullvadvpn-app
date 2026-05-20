//! TODO

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Id(uuid::Uuid);

impl Id {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Tries to parse a UUID from a raw String. If it is successful, an
    /// [`Id`] is instantiated.
    pub fn from_string(id: String) -> Option<Self> {
        use std::str::FromStr;
        uuid::Uuid::from_str(&id).ok().map(Self)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
