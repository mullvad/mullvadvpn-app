use std::{env, fs, path::PathBuf, process::Command};

#[cfg(windows)]
fn make_lang_id(p: u16, s: u16) -> u16 {
    (s << 10) | p
}

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("git-commit-date.txt"), commit_date()).unwrap();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set("ProductVersion", mullvad_version::VERSION);
        res.set_icon("../dist-assets/icon.ico");
        res.set_language(make_lang_id(
            windows_sys::Win32::System::SystemServices::LANG_ENGLISH as u16,
            windows_sys::Win32::System::SystemServices::SUBLANG_ENGLISH_US as u16,
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
        .args(["log", "-1", "--date=short", "--pretty=format:%cd"])
        .output()
        .expect("Unable to get git commit date");
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .to_owned()
}
