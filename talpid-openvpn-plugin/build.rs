fn main() {
    const PROTO_FILE: &str = "proto/openvpn_plugin.proto";
    tonic_build::compile_protos(PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={}", PROTO_FILE);

    #[cfg(windows)]
    {
        let product_version = env!("CARGO_PKG_VERSION").replacen(".0", "", 1);
        let mut res = winres::WindowsResource::new();
        res.set("ProductVersion", &product_version);
        res.set_icon("../dist-assets/icon.ico");
        res.set_language(winapi::um::winnt::MAKELANGID(
            winapi::um::winnt::LANG_ENGLISH,
            winapi::um::winnt::SUBLANG_ENGLISH_US,
        ));
        res.compile().expect("Unable to generate windows resources");
    }
}
