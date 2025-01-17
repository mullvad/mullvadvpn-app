#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::pin::Pin;
use std::task::{ready, Poll};

use reqwest::header::{HeaderValue, CONTENT_LENGTH, RANGE};
use tokio::fs::{self, File};
use tokio::io::{self, AsyncWrite, AsyncWriteExt, BufWriter};

use anyhow::Context;

/// Receiver of the current progress so far
pub trait ProgressUpdater: Send + 'static {
    /// Progress so far
    fn set_progress(&mut self, fraction_complete: f32);

    /// URL that is being downloaded
    fn set_url(&mut self, url: &str);
}

// TODO: save file to protected dir so it cannot be tampered with after verification

/// This describes how to handle files that do not match an expected size
#[derive(Debug, Clone, Copy)]
pub enum SizeHint {
    /// Fail if the resulting file does not exactly match the expected size.
    Exact(usize),
    /// Fail if the resulting file is larger than the specified limit.
    Maximum(usize),
}

/// Download `url` to `file`.
///
/// # Arguments
/// - `progress_updater` - This interface is notified of download progress.
/// - `size_hint` - File size restrictions.
pub async fn get_to_file(
    file: impl AsRef<Path>,
    url: &str,
    progress_updater: &mut impl ProgressUpdater,
    size_hint: SizeHint,
) -> anyhow::Result<()> {
    let (file, mut already_fetched_bytes) = create_or_append(file).await?;
    let mut file = BufWriter::new(file);

    let client = reqwest::Client::new();

    // Fetch content length first
    let response = client.head(url).send().await.context("HEAD failed")?;
    if !response.status().is_success() {
        return response
            .error_for_status()
            .map(|_| ())
            .context("Download failed");
    }

    let total_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .context("Missing file size")?;
    let total_size: usize = total_size.to_str()?.parse().context("invalid size")?;
    check_size_hint(size_hint, total_size)?;

    progress_updater.set_url(url);

    if total_size == already_fetched_bytes {
        progress_updater.set_progress(1.);
        return Ok(());
    }
    if already_fetched_bytes > total_size {
        // If the existing file is larger, truncate it
        file.get_mut()
            .set_len(0)
            .await
            .context("Failed to truncate existing file")?;
        already_fetched_bytes = 0;
    }

    // Fetch content, one range at a time
    let mut writer = WriterWithProgress {
        file,
        progress_updater,
        written_nbytes: already_fetched_bytes,
        total_nbytes: total_size,
    };

    for range in RangeIter::new(already_fetched_bytes, total_size) {
        let mut response = client
            .get(url)
            .header(RANGE, range)
            .send()
            .await
            .context("Failed to retrieve range")?;
        let status = response.status();
        if !status.is_success() {
            return response
                .error_for_status()
                .map(|_| ())
                .context("Download failed");
        }

        while let Some(chunk) = response.chunk().await.context("Failed to read chunk")? {
            writer
                .write_all(&chunk)
                .await
                .context("Failed to write chunk")?;
        }
    }

    writer.flush().await.context("Failed to flush")?;

    Ok(())
}

/// This function succeeds if `actual` is allowed according to the [SizeHint]. Otherwise, it
/// returns an error.
fn check_size_hint(hint: SizeHint, actual: usize) -> anyhow::Result<()> {
    match hint {
        SizeHint::Exact(expected) if actual != expected => {
            anyhow::bail!("File size mismatch: expected {expected} bytes, served {actual}")
        }
        SizeHint::Maximum(limit) if actual > limit => {
            anyhow::bail!(
                "File size exceeds limit: expected at most {limit} bytes, served {actual}"
            )
        }
        _ => Ok(()),
    }
}

async fn create_or_append(path: impl AsRef<Path>) -> io::Result<(File, usize)> {
    let file = path.as_ref();
    if file.exists() {
        #[cfg(unix)]
        let size = file.metadata().map(|meta| meta.size()).unwrap_or(0);
        #[cfg(windows)]
        let size = file.metadata().map(|meta| meta.file_size()).unwrap_or(0);
        let file = fs::OpenOptions::new().append(true).open(file).await?;
        Ok((file, usize::try_from(size).unwrap()))
    } else {
        Ok((File::create(file).await?, 0))
    }
}

/// Used to download partial content
struct RangeIter {
    current: usize,
    end: usize,
}

impl RangeIter {
    /// Number of bytes to read per range request
    const CHUNK_SIZE: usize = 512 * 1024;

    fn new(current: usize, end: usize) -> Self {
        Self { current, end }
    }
}

impl Iterator for RangeIter {
    type Item = HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            return None;
        }
        let prev = self.current;

        let read_n = self.end.saturating_sub(self.current).min(Self::CHUNK_SIZE);
        if read_n == 0 {
            return None;
        }

        self.current += read_n;

        // NOTE: Subtracting 1 because range includes final byte
        let end = self.current - 1;

        Some(HeaderValue::from_str(&format!("bytes={prev}-{end}")).expect("valid range/str"))
    }
}

struct WriterWithProgress<'a, PU: ProgressUpdater> {
    file: BufWriter<File>,
    progress_updater: &'a mut PU,
    written_nbytes: usize,
    /// Actual or estimated total number of bytes
    total_nbytes: usize,
}

impl<PU: ProgressUpdater> AsyncWrite for WriterWithProgress<'_, PU> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let file = Pin::new(&mut self.file);
        let nbytes = ready!(file.poll_write(cx, buf))?;

        let total_nbytes = self.total_nbytes;
        let total_written = self.written_nbytes + nbytes;

        self.written_nbytes = total_written;
        self.progress_updater
            .set_progress(total_written as f32 / total_nbytes as f32);

        Poll::Ready(Ok(nbytes))
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.file).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.file).poll_shutdown(cx)
    }
}
