use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
extern crate windres;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let product_version = env!("CARGO_PKG_VERSION").replacen(".0", "", 1);
    fs::write(out_dir.join("product-version.txt"), product_version).unwrap();
    fs::write(out_dir.join("git-commit-date.txt"), commit_date()).unwrap();

    #[cfg(windows)]
    {
        windres::Build::new().compile("version.rc").unwrap();
    }
}

fn commit_date() -> String {
    let output = Command::new("git")
        .args(&["log", "-1", "--date=short", "--pretty=format:%cd"])
        .output()
        .expect("Unable to get git commit date");
    ::std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .to_owned()
}
