use std::time::Duration;

use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::timeout,
};

const OBTAIN_PTY_TIMEOUT: Duration = Duration::from_secs(5);

pub struct NoPty;

/// Extract pty path from stdout
pub async fn find_pty(
    re: Regex,
    process: &mut tokio::process::Child,
    log_level: log::Level,
    log_prefix: &str,
) -> Result<String, NoPty> {
    let stdout = process.stdout.take().unwrap();
    let stdout_reader = BufReader::new(stdout);

    let (pty_path, reader) = timeout(OBTAIN_PTY_TIMEOUT, async {
        let mut lines = stdout_reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            log::log!(log_level, "{log_prefix}{line}");

            if let Some(path) = re.captures(&line).and_then(|cap| cap.get(1)) {
                return Ok((path.as_str().to_owned(), lines.into_inner()));
            }
        }

        Err(NoPty)
    })
    .await
    .map_err(|_| NoPty)??;

    process.stdout.replace(reader.into_inner());

    Ok(pty_path)
}
