fn main() {
    tonic_build::compile_protos("proto/management_interface.proto").unwrap();
}
