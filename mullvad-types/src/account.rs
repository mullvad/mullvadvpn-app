use chrono::DateTime;
use chrono::offset::Utc;

pub type AccountToken = String;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AccountData {
    pub expiry: DateTime<Utc>,
}
