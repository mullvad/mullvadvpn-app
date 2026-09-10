//! Reads `extra-lan-networks.txt` from the repository root and (re)writes
//! `talpid-types/src/net/extra_lan_nets.rs`, the Rust constant compiled into the app.
//!
//! Run after editing the text file:
//!
//! ```text
//! cargo run -p talpid-types --bin generate-extra-lan-nets
//! ```
//!
//! The talpid-types tests fail if the generated file is out of date, and the upstream
//! sync workflow runs this automatically.

use std::{fs, path::PathBuf, process::ExitCode};

use talpid_types::net::extra_lan_config::{EXTRA_LAN_NETWORKS_FILE, parse, render_rust};

fn main() -> ExitCode {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input = manifest_dir.join("..").join(EXTRA_LAN_NETWORKS_FILE);
    let output = manifest_dir
        .join("src")
        .join("net")
        .join("extra_lan_nets.rs");

    let contents = match fs::read_to_string(&input) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("error: cannot read {}: {err}", input.display());
            return ExitCode::FAILURE;
        }
    };
    let nets = match parse(&contents) {
        Ok(nets) => nets,
        Err(err) => {
            eprintln!("error: {}: {err}", input.display());
            return ExitCode::FAILURE;
        }
    };

    let rendered = render_rust(&nets);
    if fs::read_to_string(&output).ok().as_deref() == Some(rendered.as_str()) {
        println!(
            "{} is up to date ({} extra network(s))",
            output.display(),
            nets.len()
        );
        return ExitCode::SUCCESS;
    }
    if let Err(err) = fs::write(&output, rendered) {
        eprintln!("error: cannot write {}: {err}", output.display());
        return ExitCode::FAILURE;
    }
    println!(
        "wrote {} ({} extra network(s))",
        output.display(),
        nets.len()
    );
    for net in &nets {
        println!("  {net}");
    }
    ExitCode::SUCCESS
}
