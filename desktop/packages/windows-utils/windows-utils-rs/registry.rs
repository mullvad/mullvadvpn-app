use std::io;

use neon::prelude::{Context, FunctionContext};
use neon::result::JsResult;
use neon::types::{JsValue, Value};
use windows_registry::LOCAL_MACHINE;

/// Registry key under `HKEY_LOCAL_MACHINE` holding the [`DEVICE_INSTALL_DISABLED_VALUE`] value.
const DEVICE_INSTALL_PARAMETERS_KEY: &str =
    r"SYSTEM\CurrentControlSet\Services\DeviceInstall\Parameters";
/// Registry value that, when set to `1`, prevents Windows from installing new devices.
const DEVICE_INSTALL_DISABLED_VALUE: &str = "DeviceInstallDisabled";
/// OS error code for a missing registry key or value.
const ERROR_FILE_NOT_FOUND: u32 = 2;

pub fn is_device_install_disabled(mut cx: FunctionContext<'_>) -> JsResult<'_, JsValue> {
    match is_device_install_disabled_inner() {
        Ok(Some(disabled)) => Ok(cx.boolean(disabled).as_value(&mut cx)),
        Ok(None) => Ok(cx.null().as_value(&mut cx)),
        Err(err) => cx.throw_error(format!(
            "Failed to read the {DEVICE_INSTALL_DISABLED_VALUE} registry value: {err}"
        )),
    }
}

/// Returns whether the `DeviceInstallDisabled` registry value is set to `1`, i.e. whether Windows
/// refuses to install new devices.
///
/// Returns `Ok(None)` if the key or value is missing, since that means device installation is not
/// disabled. Returns an error if the registry cannot be read.
fn is_device_install_disabled_inner() -> Result<Option<bool>, io::Error> {
    let value = match LOCAL_MACHINE
        .open(DEVICE_INSTALL_PARAMETERS_KEY)
        .and_then(|key| key.get_u32(DEVICE_INSTALL_DISABLED_VALUE))
    {
        Ok(value) => value,
        // The key or value does not exist, which means device installation is not disabled.
        Err(error) if raw_os_code(&error) == ERROR_FILE_NOT_FOUND => return Ok(None),
        Err(error) => return Err(to_io_error(error)),
    };

    Ok(Some(value == 1))
}

/// The windows-registry errors carry an HRESULT. Truncate it to the first 2 bytes to get the OS
/// error code, inverting the conversion in windows-result.
fn raw_os_code(error: &windows_result::Error) -> u32 {
    (error.code().0 & 0x0000FFFF) as u32
}

fn to_io_error(error: windows_result::Error) -> io::Error {
    io::Error::from_raw_os_error(error.code().0 & 0x0000FFFF)
}
