use anyhow::{anyhow, Context, Result};
use mullvad_management_interface::MullvadProxyClient;
use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
    path::Path,
};

/// Maximum size of a settings patch. Bigger files/streams cause the read to fail.
const MAX_PATCH_BYTES: usize = 10 * 1024;

/// If source is specified, read from the provided file and send it as a settings patch to the
/// daemon. Otherwise, read the patch from standard input.
pub async fn handle(source: String) -> Result<()> {
    let json_blob = tokio::task::spawn_blocking(|| get_blob(source))
        .await
        .unwrap()?;

    let mut rpc = MullvadProxyClient::new().await?;
    rpc.apply_json_settings(json_blob)
        .await
        .context("Error applying patch")?;

    println!("Settings applied");

    Ok(())
}

fn get_blob(source: String) -> Result<String> {
    match source.as_str() {
        "-" => read_settings_from_stdin().context("Failed to read from stdin"),
        _ => read_settings_from_file(source).context("Failed to read from path: {source}"),
    }
}

/// Read settings from standard input
fn read_settings_from_stdin() -> Result<String> {
    read_settings_from_reader(BufReader::new(stdin()))
}

/// Read settings from a path
fn read_settings_from_file(path: impl AsRef<Path>) -> Result<String> {
    read_settings_from_reader(BufReader::new(File::open(path)?))
}

/// Read until EOF or until newline when the last pair of braces has been closed
fn read_settings_from_reader(mut reader: impl BufRead) -> Result<String> {
    let mut buf = [0u8; MAX_PATCH_BYTES];

    let mut was_open = false;
    let mut close_after_newline = false;
    let mut brace_count: usize = 0;
    let mut cursor_pos = 0;

    loop {
        let Some(cursor) = buf.get_mut(cursor_pos..) else {
            return Err(anyhow!(
                "Patch too long: maximum length is {MAX_PATCH_BYTES} bytes"
            ));
        };

        let prev_cursor_pos = cursor_pos;
        let read_n = reader.read(cursor)?;
        if read_n == 0 {
            // EOF
            break;
        }
        cursor_pos += read_n;

        let additional_bytes = &buf[prev_cursor_pos..cursor_pos];

        if !close_after_newline {
            for next in additional_bytes {
                match next {
                    b'{' => brace_count += 1,
                    b'}' => {
                        brace_count = brace_count.checked_sub(1).with_context(|| {
                            // exit: too many closing braces
                            "syntax error: unexpected '}'"
                        })?
                    }
                    _ => (),
                }
                was_open |= brace_count > 0;
            }
            if brace_count == 0 && was_open {
                // complete settings
                close_after_newline = true;
            }
        }

        if close_after_newline && additional_bytes.contains(&b'\n') {
            // done
            break;
        }
    }

    Ok(std::str::from_utf8(&buf[0..cursor_pos])
        .context("settings must be utf8 encoded")?
        .to_owned())
}
