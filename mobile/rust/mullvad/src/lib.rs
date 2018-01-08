  #[macro_use]
extern crate lazy_static;

#[cfg(feature = "jni")]
#[allow(non_snake_case)]
pub mod android {

  extern crate jni;

  use self::jni::JNIEnv;
  use self::jni::objects::{JClass, JString};
  use self::jni::sys::jstring;
  use std::sync::atomic::{AtomicUsize, Ordering};

  lazy_static! {
    static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
}

  #[no_mangle]
  pub unsafe extern fn Java_com_mullvad_WireGuardMock_helloWorld(env: JNIEnv, _: JClass, _name: JString) -> jstring {
    let response = format!("Hello {}!", COUNTER.fetch_add(1, Ordering::SeqCst));
    env.new_string(response).unwrap().into_inner()
  }
}