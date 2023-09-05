fn main() {
    tonic_build::compile_protos("proto/ephemeralpeer.proto").unwrap();
}
