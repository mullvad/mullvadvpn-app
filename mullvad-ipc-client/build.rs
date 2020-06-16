fn main() {
    tonic_build::compile_protos("../mullvad-daemon/proto/management_interface.proto").unwrap();
}
