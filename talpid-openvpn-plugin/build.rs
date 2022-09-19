#[cfg(windows)]
fn make_lang_id(p: u16, s: u16) -> u16 {
    (s << 10) | p
}

fn main() {
    const PROTO_FILE: &str = "proto/openvpn_plugin.proto";
    tonic_build::compile_protos(PROTO_FILE).unwrap();
    println!("cargo:rerun-if-changed={}", PROTO_FILE);

    #[cfg(windows)]
    {
        let product_version = option_env!("TALPID_PRODUCT_VERSION")
            .map(str::to_owned)
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").replacen(".0", "", 1));
        let mut res = winres::WindowsResource::new();
        res.set("ProductVersion", &product_version);
        res.set_icon("../dist-assets/icon.ico");
        res.set_language(make_lang_id(
            windows_sys::Win32::System::SystemServices::LANG_ENGLISH as u16,
            windows_sys::Win32::System::SystemServices::SUBLANG_ENGLISH_US as u16,
        ));
        res.compile().expect("Unable to generate windows resources");
    }
}
