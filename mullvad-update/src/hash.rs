//! Compute checksum for SHA-256

use anyhow::Context;
use sha2::Digest;
use tokio::io::{AsyncRead, AsyncReadExt};

/// Maximum number of bytes to read at a time
const BUF_SIZE: usize = 10 * 1024 * 1024;

/// Generate SHA256 checksum for `reader`
pub async fn checksum(mut reader: impl AsyncRead + Unpin) -> anyhow::Result<[u8; 32]> {
    let mut hasher = sha2::Sha256::new();

    // Read data into hasher
    let mut buffer = vec![0u8; BUF_SIZE];
    loop {
        let read_n = reader
            .read(&mut buffer)
            .await
            .context("Error reading bin file")?;
        if read_n == 0 {
            // We're done
            break;
        }
        hasher.update(&buffer[..read_n]);
    }

    Ok(hasher.finalize().into())
}
