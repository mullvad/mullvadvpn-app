fn main() {
    const PROTO_FILE: &str = "proto/management_interface.proto";
    tonic_build::compile_protos(PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={PROTO_FILE}");
}
