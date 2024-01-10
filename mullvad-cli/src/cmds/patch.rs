use anyhow::{Context, Result};
use mullvad_management_interface::MullvadProxyClient;
use std::{
    fs::File,
    io::{read_to_string, stdin, BufReader},
};

/// Read a settings patch and send it to the daemon for validation and
/// application.
///
/// * If `source` is "-", read the patch from standard input
/// * Otherwise, interpret `source` as a filepath and read from the provided
///   file
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

/// Output a settings patch including all currently patchable settings.
///
/// * If `source` is "-", write the patch to standard output
/// * Otherwise, interpret `source` as a filepath and write to the provided
///   file
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
