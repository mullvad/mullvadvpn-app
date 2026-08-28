use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

pub async fn forward_logs<T: AsyncRead + Unpin>(prefix: &str, stdio: T, level: log::Level) {
    let reader = BufReader::new(stdio);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log::log!(level, "{prefix}{line}");
    }
}
