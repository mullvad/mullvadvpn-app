fn main() {
    generate_grpc_code();
}

fn generate_grpc_code() {
    tonic_build::compile_protos("../talpid-openvpn-plugin/proto/openvpn_plugin.proto").unwrap();
}
