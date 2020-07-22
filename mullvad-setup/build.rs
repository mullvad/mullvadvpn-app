use std::{env, fs, path::PathBuf};

fn main() {
    const PROTO_FILE: &str = "../mullvad-daemon/proto/management_interface.proto";
    tonic_build::compile_protos(PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={}", PROTO_FILE);

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let product_version = env!("CARGO_PKG_VERSION").replacen(".0", "", 1);
    fs::write(out_dir.join("product-version.txt"), &product_version).unwrap();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set("ProductVersion", &product_version);
        res.set_language(winapi::um::winnt::MAKELANGID(
            winapi::um::winnt::LANG_ENGLISH,
            winapi::um::winnt::SUBLANG_ENGLISH_US,
        ));
        res.compile().expect("Unable to generate windows resources");
    }
}
