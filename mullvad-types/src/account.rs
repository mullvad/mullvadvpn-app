use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Serialize};

pub type AccountToken = String;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AccountData {
    pub expiry: DateTime<Utc>,
}
