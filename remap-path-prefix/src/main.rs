fn main() {
    match get_remap_path_prefix() {
        Ok(prefix) => println!("{prefix}"),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_remap_path_prefix() -> Result<String, Box<dyn std::error::Error>> {
    let cargo_home = home::cargo_home()?.display().to_string();
    let rustup_home = home::rustup_home()?.display().to_string();

    let source_dir = env!("CARGO_MANIFEST_DIR")
        .split(concat!("/", env!("CARGO_PKG_NAME")))
        .next()
        .ok_or_else(|| "Could not find Cargo build dir.".to_string())?;

    Ok(format!("--remap-path-prefix {cargo_home}=/CARGO_HOME --remap-path-prefix {rustup_home}=/RUSTUP_HOME --remap-path-prefix {source_dir}=/SOURCE_DIR"))
}
