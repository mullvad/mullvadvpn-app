use std::path::Path;
use std::pin::Pin;
use std::task::{ready, Poll};

use tokio::fs::File;
use tokio::io::{self, AsyncWrite, AsyncWriteExt, BufWriter};

use anyhow::Context;

/// Receiver of the current progress so far
pub trait ProgressUpdater: Send + 'static {
    /// Progress so far
    fn set_progress(&mut self, fraction_complete: f32);

    /// URL that is being downloaded
    fn set_url(&mut self, url: &str);
}

// TODO: handle resumed downloads
// TODO: save file to protected dir so it cannot be tampered with after verification

/// Download `url` to `file`.
///
/// # Arguments
/// - `progress_updater` - This interface is notified of download progress.
/// - `size_hint` - Assumed size if the HTTP header doesn't reveal it.
pub async fn get_to_file(
    file: impl AsRef<Path>,
    url: &str,
    progress_updater: &mut impl ProgressUpdater,
    size_hint: u64,
) -> anyhow::Result<()> {
    let file = BufWriter::new(File::create(file).await?);
    let mut get_result = reqwest::get(url).await?;
    progress_updater.set_url(url);

    let total_size = get_result.content_length().unwrap_or(size_hint);

    if !get_result.status().is_success() {
        return get_result
            .error_for_status()
            .map(|_| ())
            .context("Download failed");
    }

    let mut writer = WriterWithProgress {
        file,
        progress_updater,
        written_nbytes: 0,
        total_nbytes: total_size as usize,
    };

    while let Some(chunk) = get_result.chunk().await.context("Failed to read chunk")? {
        writer
            .write_all(&chunk)
            .await
            .context("Failed to write chunk")?;
    }

    writer.flush().await.context("Failed to flush")?;

    Ok(())
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
