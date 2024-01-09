use anyhow::{anyhow, Context, Result};
use mullvad_management_interface::MullvadProxyClient;
use std::{
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

/// Maximum size of a settings patch. Bigger files/streams cause the read to fail.
const MAX_PATCH_BYTES: usize = 10 * 1024;

/// If source is specified, read from the provided file and send it as a settings patch to the
/// daemon. Otherwise, read the patch from standard input.
pub async fn import(source: String) -> Result<()> {
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

/// If source is specified, write a patch to the file. Otherwise, write the patch to standard
/// output.
pub async fn export(dest: String) -> Result<()> {
    let mut rpc = MullvadProxyClient::new().await?;
    let json_blob = rpc
        .export_json_settings()
        .await
        .context("Error exporting patch")?;

    tokio::task::spawn_blocking(|| put_blob(dest, json_blob))
        .await
        .unwrap()?;

    Ok(())
}

fn get_blob(source: String) -> Result<String> {
    match source.as_str() {
        "-" => read_settings_from_stdin().context("Failed to read from stdin"),
        _ => {
            read_settings_from_file(&source).context(format!("Failed to read from path: {source}"))
        }
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

fn put_blob(dest: String, blob: String) -> Result<()> {
    match dest.as_str() {
        "-" => write_settings_to_stdout(blob).context("Failed to write to stdout"),
        _ => write_settings_to_file(&dest, blob).context(format!("Failed to write to path {dest}")),
    }
}

/// Write patch to standard output
fn write_settings_to_stdout(blob: String) -> Result<()> {
    write_settings_using_writer(BufWriter::new(stdout()), blob)
}

/// Write patch to path
fn write_settings_to_file(path: impl AsRef<Path>, blob: String) -> Result<()> {
    write_settings_using_writer(
        BufWriter::new(File::options().create(true).write(true).open(path)?),
        blob,
    )
}

fn write_settings_using_writer(mut writer: impl Write, blob: String) -> Result<()> {
    writer
        .write_all(blob.as_bytes())
        .context("Failed to write blob to destination")?;
    writer.write_all(b"\n").context("Failed to write newline")
}
