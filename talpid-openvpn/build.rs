fn main() {
    generate_grpc_code();
}

fn generate_grpc_code() {
    const PROTO_FILE: &str = "../talpid-openvpn-plugin/proto/openvpn_plugin.proto";
    tonic_build::compile_protos(PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={PROTO_FILE}");
}
