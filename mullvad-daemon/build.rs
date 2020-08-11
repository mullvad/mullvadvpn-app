use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let product_version = env!("CARGO_PKG_VERSION").replacen(".0", "", 1);
    fs::write(out_dir.join("product-version.txt"), &product_version).unwrap();
    fs::write(out_dir.join("git-commit-date.txt"), commit_date()).unwrap();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set("ProductVersion", &product_version);
        res.set_icon("../dist-assets/icon.ico");
        res.set_language(winapi::um::winnt::MAKELANGID(
            winapi::um::winnt::LANG_ENGLISH,
            winapi::um::winnt::SUBLANG_ENGLISH_US,
        ));
        println!("cargo:rerun-if-env-changed=MULLVAD_ADD_MANIFEST");
        if env::var("MULLVAD_ADD_MANIFEST")
            .map(|s| s != "0")
            .unwrap_or(false)
        {
            res.set_manifest_file("mullvad-daemon.manifest");
        } else {
            println!("cargo:warning=Skipping mullvad-daemon manifest");
        }
        res.compile().expect("Unable to generate windows resources");
    }
}

fn commit_date() -> String {
    let output = Command::new("git")
        .args(&["log", "-1", "--date=short", "--pretty=format:%cd"])
        .output()
        .expect("Unable to get git commit date");
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .to_owned()
}
