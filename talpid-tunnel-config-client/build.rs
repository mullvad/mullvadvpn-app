fn main() {
    tonic_prost_build::configure()
        .with_extended_rust_types(true)
        .compile_protos(&["proto/ephemeralpeer.proto"], &["proto/"])
        .unwrap();
}
