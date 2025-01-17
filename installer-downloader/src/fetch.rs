use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

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
pub fn get_to_file(
    file: impl AsRef<Path>,
    url: &str,
    progress_updater: &mut impl ProgressUpdater,
    size_hint: u64,
) -> anyhow::Result<()> {
    let file = BufWriter::new(File::create(file)?);
    let mut get_result = reqwest::blocking::get(url)?;
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

    get_result.copy_to(&mut writer)?;
    Ok(())
}

struct WriterWithProgress<'a, PU: ProgressUpdater> {
    file: BufWriter<File>,
    progress_updater: &'a mut PU,
    written_nbytes: usize,
    /// Actual or estimated total number of bytes
    total_nbytes: usize,
}

impl<PU: ProgressUpdater> std::io::Write for WriterWithProgress<'_, PU> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let nbytes = self.file.write(buf)?;

        self.written_nbytes += nbytes;
        self.progress_updater
            .set_progress((self.written_nbytes as f32 / self.total_nbytes as f32));

        Ok(nbytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
