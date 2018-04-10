use chrono::offset::Utc;
use chrono::DateTime;

pub type AccountToken = String;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AccountData {
    pub expiry: DateTime<Utc>,
}
