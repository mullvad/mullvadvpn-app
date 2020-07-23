fn main() {
    const OPENVPN_PROTO_FILE: &str = "../talpid-openvpn-plugin/proto/openvpn_plugin.proto";
    tonic_build::compile_protos(OPENVPN_PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={}", OPENVPN_PROTO_FILE);
}
