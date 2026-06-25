fn main() {
    // Compile both proto files together so they can reference each other
    tonic_prost_build::configure()
        .with_extended_rust_types(true)
        .compile_protos(
            &[
                "proto/management_interface.proto",
                "proto/relay_selector.proto",
            ],
            &["proto/"],
        )
        .unwrap();
}
