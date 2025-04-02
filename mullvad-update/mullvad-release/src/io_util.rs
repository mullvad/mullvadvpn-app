//! File and I/O utilities

use std::path::Path;

use anyhow::Context;
use tokio::fs;

/// Wait for user to respond with yes or no
/// This returns `false` if reading from stdin fails
pub async fn wait_for_confirm(prompt: &str) -> bool {
    const DEFAULT: bool = true;

    let prompt = prompt.to_owned();

    tokio::task::spawn_blocking(move || {
        let stdin = std::io::stdin();

        loop {
            let mut s = String::new();

            print!("{prompt}");
            if DEFAULT {
                println!(" [Y/n]");
            } else {
                println!(" [y/N]");
            }

            stdin.read_line(&mut s).context("Failed to read line")?;

            match s.trim().to_ascii_lowercase().as_str() {
                "" => break Ok::<bool, anyhow::Error>(DEFAULT),
                "y" | "ye" | "yes" => break Ok(true),
                "n" | "no" => break Ok(false),
                _ => (),
            }
        }
    })
    .await
    .unwrap()
    .unwrap_or(false)
}

/// Wait for user to respond with any input, ignoring empty responses
pub async fn wait_for_input(prompt: &str) -> anyhow::Result<String> {
    let prompt = prompt.to_owned();
    tokio::task::spawn_blocking(move || {
        let stdin = std::io::stdin();

        loop {
            let mut s = String::new();

            println!("{prompt}");

            stdin.read_line(&mut s).context("Failed to read line")?;

            match s.trim().to_ascii_lowercase().as_str() {
                "" => continue,
                input => break Ok(input.to_owned()),
            }
        }
    })
    .await
    .unwrap()
}

/// Recursively create directories and write to 'file'
pub async fn create_dir_and_write(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> anyhow::Result<()> {
    let path = path.as_ref();

    let parent_dir = path.parent().context("Missing parent directory")?;
    fs::create_dir_all(parent_dir)
        .await
        .context("Failed to create directories")?;

    fs::write(path, contents)
        .await
        .with_context(|| format!("Failed to write to {}", path.display()))?;
    Ok(())
}
