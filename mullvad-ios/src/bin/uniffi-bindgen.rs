//! Host-side entry point for generating uniffi bindings (Swift) for the gotatun
//! FFI. Invoked in library mode against the compiled `libmullvad_ios` staticlib;
//! see `ios/generate-gotatun-bindings.sh`.

fn main() {
    #[cfg(target_os = "macos")]
    {
        // `uniffi_bindgen_main` only exists with the `cli` feature. Gate the body so the
        // binary still compiles during a normal `cargo test`/`cargo build` (which builds
        // all targets) without that feature enabled.
        #[cfg(feature = "uniffi-cli")]
        uniffi::uniffi_bindgen_main();
        #[cfg(not(feature = "uniffi-cli"))]
        eprintln!("rebuild with `--features uniffi-cli` to run uniffi-bindgen");
    }
}
