use jnix::jni::objects::{JObject, JString};
use std::ops::Deref;

pub trait IsNull {
    fn is_null(&self) -> bool;
}

impl IsNull for JObject<'_> {
    fn is_null(&self) -> bool {
        self.deref().is_null()
    }
}

impl IsNull for JString<'_> {
    fn is_null(&self) -> bool {
        self.deref().is_null()
    }
}
