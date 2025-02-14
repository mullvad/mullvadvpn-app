//! A downloader that supports range requests and resuming downloads

use std::{
    path::Path,
    pin::Pin,
    task::{ready, Poll},
};

use reqwest::header::{HeaderValue, CONTENT_LENGTH, RANGE};
use tokio::{
    fs::{self, File},
    io::{self, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufWriter},
};

use anyhow::Context;

/// Receiver of the current progress so far
pub trait ProgressUpdater: Send + 'static {
    /// Progress so far
    fn set_progress(&mut self, fraction_complete: f32);

    /// URL that is being downloaded
    fn set_url(&mut self, url: &str);
}

/// This describes how to handle files that do not match an expected size
#[derive(Debug, Clone, Copy)]
pub enum SizeHint {
    /// Fail if the resulting file does not exactly match the expected size.
    Exact(usize),
    /// Fail if the resulting file is larger than the specified limit.
    Maximum(usize),
}

/// Download `url` to `file`. If the file already exists, this appends to it, as long
/// as the file pointed to by `url` is larger than it.
///
/// Make sure that `file` is stored in a secure directory.
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
    let file = create_or_append(file).await?;
    let file = BufWriter::new(file);
    get_to_writer(file, url, progress_updater, size_hint).await
}

/// Download `url` to `writer`.
///
/// # Arguments
/// - `progress_updater` - This interface is notified of download progress.
/// - `size_hint` - File size restrictions.
pub async fn get_to_writer(
    mut writer: impl AsyncWrite + AsyncSeek + Unpin,
    url: &str,
    progress_updater: &mut impl ProgressUpdater,
    size_hint: SizeHint,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    progress_updater.set_url(url);
    progress_updater.set_progress(0.);

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

    let already_fetched_bytes = writer
        .stream_position()
        .await
        .context("failed to get existing file size")?
        .try_into()
        .context("invalid size")?;

    if total_size == already_fetched_bytes {
        progress_updater.set_progress(1.);
        return Ok(());
    }
    if already_fetched_bytes > total_size {
        anyhow::bail!("Found existing file that was larger");
    }

    // Fetch content, one range at a time
    let mut writer = WriterWithProgress {
        writer,
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

        let mut bytes_read = 0;

        while let Some(chunk) = response.chunk().await.context("Failed to read chunk")? {
            bytes_read += chunk.len();
            if bytes_read > RangeIter::CHUNK_SIZE {
                // Protect against servers responding with more data than expected
                anyhow::bail!("Server returned more than chunk-sized bytes");
            }

            writer
                .write_all(&chunk)
                .await
                .context("Failed to write chunk")?;
        }
    }

    writer.shutdown().await.context("Failed to flush")?;

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

/// If a file exists, append to it. Otherwise, create a new file
async fn create_or_append(path: impl AsRef<Path>) -> io::Result<File> {
    match fs::File::create_new(&path).await {
        // New file created
        Ok(file) => Ok(file),
        // Append to an existing file
        Err(_err) => {
            let mut file = fs::OpenOptions::new().append(true).open(path).await?;
            // Seek to end, or else the seek position might be wrong
            file.seek(io::SeekFrom::End(0)).await?;
            Ok(file)
        }
    }
}

/// Used to download partial content
struct RangeIter {
    current: usize,
    end: usize,
}

impl RangeIter {
    /// Number of bytes to read per range request
    pub const CHUNK_SIZE: usize = 512 * 1024;

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

struct WriterWithProgress<'a, PU: ProgressUpdater, Writer> {
    writer: Writer,
    progress_updater: &'a mut PU,
    written_nbytes: usize,
    /// Actual or estimated total number of bytes
    total_nbytes: usize,
}

impl<PU: ProgressUpdater, Writer: AsyncWrite + Unpin> AsyncWrite
    for WriterWithProgress<'_, PU, Writer>
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let file = Pin::new(&mut self.writer);
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
        Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.writer).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use async_tempfile::TempDir;
    use rand::RngCore;
    use tokio::{fs, io::AsyncWriteExt};

    use super::*;

    #[tokio::test]
    async fn test_create_or_append() -> anyhow::Result<()> {
        let temp_dir = TempDir::new().await?;
        let file_path = temp_dir.join("test");

        // Write to a new file
        const CONTENT: &[u8] = b"very important file";

        let mut file = create_or_append(&file_path).await?;
        file.write_all(CONTENT).await?;
        file.flush().await?;
        drop(file);

        assert_eq!(fs::read(&file_path).await?, CONTENT);

        // Verify that we can trust the stream position
        let mut file = create_or_append(&file_path).await?;
        let content_len: u64 = CONTENT.len().try_into()?;
        assert_eq!(file.stream_position().await?, content_len);
        drop(file);

        // Append some more stuff
        const EXTRA: &[u8] = b"my addition";

        let mut file = create_or_append(&file_path).await?;
        file.write_all(EXTRA).await?;
        file.flush().await?;
        drop(file);

        // Append occurred correctly
        const COMPLETE_STRING: &[u8] = b"very important filemy addition";
        assert_eq!(fs::read(file_path).await?, COMPLETE_STRING);

        Ok(())
    }

    #[derive(Default)]
    struct FakeProgressUpdater {
        complete: f32,
        url: String,
    }

    impl ProgressUpdater for FakeProgressUpdater {
        fn set_progress(&mut self, fraction_complete: f32) {
            self.complete = fraction_complete;
        }

        fn set_url(&mut self, url: &str) {
            self.url = url.to_owned();
        }
    }

    /// Test that [get_to_writer] correctly downloads new files
    #[tokio::test]
    async fn test_fetch_complete() -> anyhow::Result<()> {
        // Generate random data
        let file_data = Box::leak(Box::new(vec![0u8; 1024 * 1024 + 1]));
        rand::thread_rng().fill_bytes(file_data);

        // Start server
        let mut server = mockito::Server::new_async().await;
        let file_url = format!("{}/my_file", server.url());
        add_file_server_mock(&mut server, "/my_file", file_data);

        // Download the file to `writer` and compare it to `file_data`
        let mut writer = Cursor::new(vec![]);
        let mut progress_updater = FakeProgressUpdater::default();

        get_to_writer(
            &mut writer,
            &file_url,
            &mut progress_updater,
            SizeHint::Exact(file_data.len()),
        )
        .await
        .context("Complete download failed")?;

        assert_eq!(progress_updater.url, file_url);
        assert_eq!(progress_updater.complete, 1.);
        assert_eq!(&mut writer.into_inner(), file_data);

        Ok(())
    }

    /// Test that [get_to_writer] correctly downloads partial files
    #[tokio::test]
    async fn test_fetch_interrupted() -> anyhow::Result<()> {
        // Generate random data
        let file_data = Box::leak(Box::new(vec![0u8; 1024 * 1024]));
        rand::thread_rng().fill_bytes(file_data);

        // Start server
        let mut server = mockito::Server::new_async().await;
        let file_url = format!("{}/my_file", server.url());
        add_file_server_mock(&mut server, "/my_file", file_data);

        // Interrupt after exactly half the file has been downloaded
        let mut limited_buffer = vec![0u8; file_data.len() / 2].into_boxed_slice();
        let mut writer = Cursor::new(&mut limited_buffer[..]);

        let mut progress_updater = FakeProgressUpdater::default();

        get_to_writer(
            &mut writer,
            &file_url,
            &mut progress_updater,
            SizeHint::Exact(file_data.len()),
        )
        .await
        .expect_err("Expected interrupted download");

        assert_eq!(progress_updater.url, file_url);

        let completed = progress_updater.complete;
        assert!(
            (completed - 0.5).abs() < f32::EPSILON,
            "expected half to be completed, got {completed}"
        );

        assert_eq!(
            &*limited_buffer,
            &file_data[..limited_buffer.len()],
            "partial download incorrect"
        );

        // Download the remainder
        let writer = limited_buffer.into_vec();
        let partial_len = writer.len();
        let mut writer = Cursor::new(writer);
        writer.set_position(partial_len as u64);

        let mut progress_updater = FakeProgressUpdater::default();

        get_to_writer(
            &mut writer,
            &file_url,
            &mut progress_updater,
            SizeHint::Exact(file_data.len()),
        )
        .await
        .context("Partial download failed")?;

        assert_eq!(progress_updater.url, file_url);
        assert_eq!(progress_updater.complete, 1.);
        assert_eq!(&mut writer.into_inner(), file_data);

        Ok(())
    }

    /// Create endpoints that serve a file at `url_path` using range requests
    fn add_file_server_mock(server: &mut mockito::Server, url_path: &str, data: &'static [u8]) {
        // Respond to head requests with file size
        server
            .mock("HEAD", url_path)
            .with_header(CONTENT_LENGTH, &data.len().to_string())
            .create();

        // Respond to range requests with file
        server
            .mock("GET", url_path)
            .with_body_from_request(|request| {
                let range = request.header(RANGE);
                let range = range[0].to_str().expect("expected str");
                let (begin, end) = parse_http_range(range).expect("invalid range");

                data[begin..=end].to_vec()
            })
            .create();
    }

    /// Parse a range header value, e.g. "bytes=0-31"
    fn parse_http_range(val: &str) -> anyhow::Result<(usize, usize)> {
        // parse: bytes=0-31
        let (_, val) = val.split_once('=').context("invalid range header")?;
        let (begin, end) = val.split_once('-').context("invalid range")?;

        let begin: usize = begin.parse().context("invalid range begin")?;
        let end: usize = end.parse().context("invalid range end")?;

        Ok((begin, end))
    }

    /// Make sure unexpectedly large files are rejected
    #[tokio::test]
    async fn test_nefarious_sizes() -> anyhow::Result<()> {
        // Head length is too large
        let mut server = mockito::Server::new_async().await;
        let file_url = format!("{}/my_file", server.url());
        server
            .mock("HEAD", "/my_file")
            .with_header(CONTENT_LENGTH, "2")
            .create();

        get_to_writer(
            Cursor::new(vec![]),
            &file_url,
            &mut FakeProgressUpdater::default(),
            SizeHint::Exact(1),
        )
        .await
        .expect_err("Reject unexpected content length");

        // Malicious range response
        // Serve the entire file rather than the requested range
        let file_data = vec![0u8; 2 * RangeIter::CHUNK_SIZE];

        let mut server = mockito::Server::new_async().await;
        let file_url = format!("{}/my_file", server.url());
        server
            .mock("HEAD", "/my_file")
            .with_header(CONTENT_LENGTH, &file_data.len().to_string())
            .create();
        server
            .mock("GET", "/my_file")
            .with_body(&file_data)
            .create();

        get_to_writer(
            Cursor::new(vec![]),
            &file_url,
            &mut FakeProgressUpdater::default(),
            SizeHint::Exact(file_data.len()),
        )
        .await
        .expect_err("Reject unexpected chunk sizes");

        Ok(())
    }
}
