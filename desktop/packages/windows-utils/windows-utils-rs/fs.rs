use std::io;
use std::path::Path;

use neon::prelude::{Context, FunctionContext};
use neon::result::JsResult;
use neon::types::{JsString, JsValue, Value};

use talpid_error::ErrorExt;

use windows::Win32::Foundation::ERROR_PIPE_BUSY;
use windows::Win32::System::Pipes::WaitNamedPipeW;
use windows::core::HSTRING;

/// Maximum time to wait for an instance of the pipe to become available.
const PIPE_TIMEOUT_MSEC: u32 = 5000;

/// Number of times to retry opening the pipe if it is busy.
const PIPE_BUSY_RETRIES: u32 = 3;

#[derive(thiserror::Error, Debug)]
enum Error {
    /// Failed to open the provided file
    #[error("Failed to open named pipe")]
    OpenPipe(#[source] io::Error),

    /// Failed to check pipe ownership (GetSecurityInfo)
    #[error("Failed to check named pipe ownership (GetSecurityInfo failed)")]
    CheckPermissions(#[source] io::Error),

    /// Failed to wait on named pipe
    #[error("Failed to wait on named pipe")]
    WaitPipe(#[source] io::Error),
}

pub fn pipe_is_admin_owned(mut cx: FunctionContext<'_>) -> JsResult<'_, JsValue> {
    let link_path = cx.argument::<JsString>(0)?.value(&mut cx);

    match pipe_is_admin_owned_inner(link_path) {
        Ok(is_admin_owned) => Ok(cx.boolean(is_admin_owned).as_value(&mut cx)),
        Err(err) => cx.throw_error(err.display_chain()),
    }
}

/// If the pipe is busy, this blocks for at most [PIPE_BUSY_RETRIES] * [PIPE_TIMEOUT_MSEC] ms.
fn pipe_is_admin_owned_inner<P: AsRef<Path>>(path: P) -> Result<bool, Error> {
    let path = path.as_ref();

    let mut retries_left = PIPE_BUSY_RETRIES;
    let client = loop {
        match std::fs::File::options().read(true).open(path) {
            // If the pipe is busy, wait for it to become available and try again
            Err(err)
                if retries_left > 0 && err.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) =>
            {
                retries_left -= 1;
                wait_named_pipe(path).map_err(Error::WaitPipe)?;
            }
            result => break result.map_err(Error::OpenPipe)?,
        }
    };

    talpid_windows::fs::is_admin_owned(client).map_err(Error::CheckPermissions)
}

/// Wait for an instance of the pipe to become available. This blocks for at most
/// [PIPE_TIMEOUT_MSEC] ms.
///
/// <https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew>
fn wait_named_pipe(pipe_name: &Path) -> io::Result<()> {
    let pipe_name = HSTRING::from(pipe_name);
    // SAFETY: `pipe_name` is a valid, null-terminated wide string.
    let status = unsafe { WaitNamedPipeW(&pipe_name, PIPE_TIMEOUT_MSEC) };
    if !status.as_bool() {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
