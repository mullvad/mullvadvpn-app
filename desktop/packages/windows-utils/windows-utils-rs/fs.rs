use std::io;
use std::path::Path;

use neon::prelude::{Context, FunctionContext};
use neon::result::JsResult;
use neon::types::{JsString, JsValue, Value};

use talpid_error::ErrorExt;

use widestring::{U16CStr, U16CString};

use windows_sys::Win32::Foundation::ERROR_PIPE_BUSY;
use windows_sys::Win32::System::Pipes::WaitNamedPipeW;

const PIPE_TIMEOUT_MSEC: u32 = 5000;

#[derive(thiserror::Error, Debug)]
enum Error {
    /// Failed to open the provided file
    #[error("Failed to open named pipe")]
    OpenPipe(#[source] io::Error),

    /// Failed to check pipe ownership (GetSecurityInfo)
    #[error("Failed to check named pipe ownership (GetSecurityInfo failed)")]
    CheckPermissions(#[source] io::Error),

    /// Failed to wait on named pipe
    #[error("Timed out waiting on named pipe")]
    PipeTimeout(#[source] io::Error),
}

pub fn pipe_is_admin_owned(mut cx: FunctionContext<'_>) -> JsResult<'_, JsValue> {
    let link_path = cx.argument::<JsString>(0)?.value(&mut cx);

    match pipe_is_admin_owned_inner(link_path) {
        Ok(is_admin_owned) => Ok(cx.boolean(is_admin_owned).as_value(&mut cx)),
        Err(err) => cx.throw_error(err.display_chain()),
    }
}

fn pipe_is_admin_owned_inner<P: AsRef<Path>>(path: P) -> Result<bool, Error> {
    let path = path.as_ref();

    let client = loop {
        let result = std::fs::File::options().read(true).open(path);

        match result {
            Ok(client) => break client,
            // If the pipe is busy, wait for it to become available
            Err(err) if err.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) => {
                let pipe_name = U16CString::from_os_str_truncate(path);
                wait_named_pipe(&pipe_name).map_err(Error::PipeTimeout)?;
                // try again
            }
            Err(err) => return Err(Error::OpenPipe(err)),
        }
    };

    talpid_windows::fs::is_admin_owned(client).map_err(Error::CheckPermissions)
}

/// If an instance of the pipe is available before the [time-out interval](PIPE_TIMEOUT_MSEC)
/// elapses, the return value is `Ok(())`, otherwise `Err(_)`.
///
/// <https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew>
fn wait_named_pipe(pipe_name: &U16CStr) -> io::Result<()> {
    // SAFETY: `pipe_name` is null-terminated.
    let status = unsafe { WaitNamedPipeW(pipe_name.as_ptr(), PIPE_TIMEOUT_MSEC) };
    if status == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
