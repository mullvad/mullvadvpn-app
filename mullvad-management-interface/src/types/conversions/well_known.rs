use crate::types;

impl From<bool> for types::BoolValue {
    fn from(data: bool) -> Self {
        types::BoolValue {
            value: data,
        }
    }
}

impl From<String> for types::StringValue {
    fn from(data: String) -> Self {
        types::StringValue {
            value: data,
        }
    }
}