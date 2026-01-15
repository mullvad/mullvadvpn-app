use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")?;
    println!("cargo::rustc-env=CARGO_CFG_TARGET_ARCH={target_arch}");
    Ok(())
}
