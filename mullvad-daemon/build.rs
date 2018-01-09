// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;


fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    File::create(out_dir.join("git-commit-info.txt"))
        .unwrap()
        .write_all(commit_info().as_bytes())
        .unwrap();
}

// Implementation borrowed from rustfmt. Returns a string containing commit hash and commit date
// if it was able to obtain it, otherwise an empty string.
fn commit_info() -> String {
    match (commit_description(), commit_date()) {
        (Some(hash), Some(date)) => format!("{} {}", hash.trim(), date),
        _ => String::new(),
    }
}

fn commit_description() -> Option<String> {
    Command::new("git")
        .args(&["describe", "--dirty"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
}

fn commit_date() -> Option<String> {
    Command::new("git")
        .args(&["log", "-1", "--date=short", "--pretty=format:%cd"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
}
