use jnix::jni::{objects::GlobalRef, JavaVM};

pub struct AndroidContext {
    pub jvm: JavaVM,
    pub vpn_service: GlobalRef,
}

impl Clone for AndroidContext {
    fn clone(&self) -> Self {
        let jvm_pointer = self.jvm.get_java_vm_pointer();
        let jvm =
            unsafe { JavaVM::from_raw(jvm_pointer).expect("Failed to get pointer to Java VM") };

        AndroidContext {
            jvm,
            vpn_service: self.vpn_service.clone(),
        }
    }
}
