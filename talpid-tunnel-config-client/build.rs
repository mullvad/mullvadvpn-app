fn main() {
    tonic_build::compile_protos("proto/ephemeralpeer.proto").unwrap();
    match std::env::var("TARGET").unwrap().as_str() {
        "aarch64-apple-ios" | "aarch64-apple-ios-sim" => {
            let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            cbindgen::Builder::new()
                .with_crate(crate_dir)
                .with_language(cbindgen::Language::C)
                .generate()
                .expect("failed to generate bindings")
                .write_to_file("../ios/MullvadPostQuantum/talpid-tunnel-config-client/include/talpid_tunnel_config_client.h");
        }
        &_ => (),
    }
}
