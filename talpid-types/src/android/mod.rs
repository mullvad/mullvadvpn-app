use jnix::jni::{objects::GlobalRef, JavaVM};
use std::sync::Arc;

#[derive(Clone)]
pub struct AndroidContext {
    pub jvm: Arc<JavaVM>,
    pub vpn_service: GlobalRef,
}
