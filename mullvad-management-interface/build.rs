fn main() {
    let fds = protox::compile(
        ["management_interface.proto", "relay_selector.proto"],
        ["proto/"],
    )
    .unwrap();

    // Compile both proto files together so they can reference each other
    tonic_prost_build::configure()
        .with_extended_rust_types(true)
        .compile_fds(fds)
        .unwrap();
}
