fn main() {
    tonic_build::compile_protos("proto/tunnel_config.proto").unwrap();
}
