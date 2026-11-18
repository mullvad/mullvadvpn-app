//! Host-side entry point for generating uniffi bindings (Swift)
//! FFI. Invoked in library mode against the compiled `libmullvad_ios` staticlib;
//! see `ios/generate-uniffi-bindings.sh`.

fn main() {
    #[cfg(target_os = "macos")]
    uniffi::uniffi_bindgen_main();
}
