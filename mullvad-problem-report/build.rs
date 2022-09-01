use std::{env, fs, path::PathBuf};

#[cfg(windows)]
fn make_lang_id(p: u16, s: u16) -> u16 {
    (s << 10) | p
}

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let product_version = env!("CARGO_PKG_VERSION").replacen(".0", "", 1);
    fs::write(out_dir.join("product-version.txt"), &product_version).unwrap();

    #[cfg(windows)]
    {
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
