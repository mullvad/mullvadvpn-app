use jni::objects::{JObject, JString};
use std::ops::Deref;

pub trait IsNull {
    fn is_null(&self) -> bool;
}

impl<'a> IsNull for JObject<'a> {
    fn is_null(&self) -> bool {
        self.deref().is_null()
    }
}

impl<'a> IsNull for JString<'a> {
    fn is_null(&self) -> bool {
        self.deref().is_null()
    }
}
