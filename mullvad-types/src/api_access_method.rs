use serde::{Deserialize, Serialize};

/// Daemon settings for API access methods.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub api_access_methods: Vec<AccessMethod>,
}

/// API access method datastructure.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessMethod {
    pub name: String,
}

impl Settings {
    // TODO: Do I have to clone?
    pub fn get_access_methods(&self) -> Vec<AccessMethod> {
        self.api_access_methods.clone()
    }
}
