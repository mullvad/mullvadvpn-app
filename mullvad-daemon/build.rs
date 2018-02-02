use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;


fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    File::create(out_dir.join("git-commit-desc.txt"))
        .unwrap()
        .write_all(commit_description().as_bytes())
        .unwrap();
    File::create(out_dir.join("git-commit-date.txt"))
        .unwrap()
        .write_all(commit_date().as_bytes())
        .unwrap();
}

fn commit_description() -> String {
    let output = Command::new("git")
        .args(&["describe", "--dirty"])
        .output()
        .expect("Unable to get git commit description");
    ::std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .to_owned()
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
