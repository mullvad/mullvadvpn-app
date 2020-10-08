use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let product_version = env!("CARGO_PKG_VERSION").replacen(".0", "", 1);
    fs::write(out_dir.join("product-version.txt"), &product_version).unwrap();
}
