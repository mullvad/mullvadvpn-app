fn main() {
    const PROTO_FILE: &str = "proto/feature.proto";
    tonic_build::compile_protos(PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={}", PROTO_FILE);
}
