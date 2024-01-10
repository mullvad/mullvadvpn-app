use anyhow::{Context, Result};
use mullvad_management_interface::MullvadProxyClient;
use std::{
    fs::File,
    io::{read_to_string, stdin, BufReader},
};

/// If source is specified (not "-"), read from the provided file and send it as a settings patch to
/// the daemon. Otherwise, read the patch from standard input.
pub async fn import(source: String) -> Result<()> {
    let json_blob = tokio::task::spawn_blocking(move || match source.as_str() {
        "-" => read_to_string(BufReader::new(stdin())).context("Failed to read from stdin"),
        _ => read_to_string(File::open(&source)?)
            .context(format!("Failed to read from path: {source}")),
    })
    .await
    .unwrap()?;

    let mut rpc = MullvadProxyClient::new().await?;
    rpc.apply_json_settings(json_blob)
        .await
        .context("Error applying patch")?;

    println!("Settings applied");

    Ok(())
}

/// If source is specified (not "-"), write a patch to the file. Otherwise, write the patch to
/// standard output.
pub async fn export(dest: String) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;
    let blob = rpc
        .export_json_settings()
        .await
        .context("Error exporting patch")?;

    match dest.as_str() {
        "-" => {
            println!("{blob}");
            Ok(())
        }
        _ => tokio::fs::write(&dest, blob)
            .await
            .context(format!("Failed to write to path {dest}")),
    }
}
