use std::io;
use std::path::Path;

use neon::prelude::{Context, FunctionContext};
use neon::result::JsResult;
use neon::types::{JsString, JsValue, Value};

#[derive(thiserror::Error, Debug)]
enum Error {
    /// Failed to open the provided file
    #[error("Failed to open named pipe")]
    OpenPipe(#[source] io::Error),

    /// Failed to check pipe ownership (GetSecurityInfo)
    #[error("Failed to check named pipe ownership (GetSecurityInfo failed)")]
    CheckPermissions(#[source] io::Error),
}

pub fn pipe_is_admin_owned(mut cx: FunctionContext<'_>) -> JsResult<'_, JsValue> {
    let link_path = cx.argument::<JsString>(0)?.value(&mut cx);

    match pipe_is_admin_owned_inner(link_path) {
        Ok(is_admin_owned) => Ok(cx.boolean(is_admin_owned).as_value(&mut cx)),
        Err(err) => cx.throw_error(format!("Failed to get pipe ownership: {err}")),
    }
}

fn pipe_is_admin_owned_inner<P: AsRef<Path>>(path: P) -> Result<bool, Error> {
    let client = std::fs::File::options()
        .read(true)
        .open(path)
        .map_err(Error::OpenPipe)?;

    talpid_windows::fs::is_admin_owned(client).map_err(Error::CheckPermissions)
}
