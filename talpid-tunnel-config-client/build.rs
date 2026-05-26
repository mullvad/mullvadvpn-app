fn main() {
    let fds = protox::compile(["ephemeralpeer.proto"], ["proto/"]).unwrap();
    tonic_prost_build::configure().compile_fds(fds).unwrap();
}
