#![cfg(target_os = "windows")]

use neon::{prelude::ModuleContext, result::NeonResult};

mod fs;
mod registry;
mod shortcut;

#[neon::main]
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
    cx.export_function("readShortcut", shortcut::read_shortcut)?;
    cx.export_function("pipeIsAdminOwned", fs::pipe_is_admin_owned)?;
    cx.export_function(
        "isDeviceInstallDisabled",
        registry::is_device_install_disabled,
    )?;

    Ok(())
}
