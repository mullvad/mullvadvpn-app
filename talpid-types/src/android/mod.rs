use jnix::jni::{objects::GlobalRef, JavaVM};

pub struct AndroidContext {
    pub jvm: JavaVM,
    pub vpn_service: GlobalRef,
}
