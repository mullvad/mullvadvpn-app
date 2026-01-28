#![cfg(target_os = "windows")]

use neon::prelude::*;
use neon::result::NeonResult;
use neon::types::JsUndefined;

mod fs;
mod shortcut;

#[neon::main]
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
    cx.export_function("readShortcut", shortcut::read_shortcut)?;
    cx.export_function("pipeIsAdminOwned", fs::pipe_is_admin_owned)?;

    cx.export_function("vmwareBorkedMyMachine", did_vmware_bork_my_machine)?;
    cx.export_function("unborkMyMachineFromVmware", unbork_my_machine_from_vmware)?;

    Ok(())
}

// TODO: Decide where to put these functions.

/// Check for the existence of the registry key `HKLM\SOFTWARE\Classes\CLSID{3d09c1ca-2bcc-40b7-b9bb-3f3ec143a87b}`,
/// which suggests that VMWare did not install correctly.
///
/// See these external resources for why this could be a problem:
/// * https://communities.vmware.com/t5/VMware-Workstation-Player/Unable-to-uninstall-VMware-Bridge-Protocol/td-p/2683023
/// * https://www.reddit.com/r/vmware/comments/18wumeh/error_code_56_on_networks_adapters_vmware_player/
fn did_vmware_bork_my_machine(mut cx: FunctionContext<'_>) -> JsResult<'_, JsBoolean> {
    match get_3d09c1ca() {
        // The key exists
        Ok(_) => Ok(cx.boolean(true)),
        // TODO: Check all error conditions
        // The key does not exists, so it is less likely that vmware cause issues
        Err(_) => Ok(cx.boolean(false)),
    }
}

/// Try to remove lingering VMWare artifacts that may cause conflicts with Mullvad VPN.
/// See [`did_vmware_bork_my_machine`] for context.
///
/// Returns successfully if the offending key was removed and returns an error if the key couldn't be removed.
fn unbork_my_machine_from_vmware(mut cx: FunctionContext<'_>) -> JsResult<'_, JsUndefined> {
    match nuke_3d09c1ca() {
        Ok(_) => Ok(cx.undefined()),
        Err(err) => cx.throw_error(err.to_string()),
    }
}

/// Check for the existence of the registry key `HKLM\SOFTWARE\Classes\CLSID{3d09c1ca-2bcc-40b7-b9bb-3f3ec143a87b}`,
/// which suggests that VMWare did not install correctly.
fn get_3d09c1ca() -> std::io::Result<winreg::reg_key::RegKey> {
    use winreg::{enums::*, reg_key::RegKey};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey(r#"SOFTWARE\Classes\CLSID{3d09c1ca-2bcc-40b7-b9bb-3f3ec143a87b}"#)
}

/// Try to remove the registry key `HKLM\SOFTWARE\Classes\CLSID{3d09c1ca-2bcc-40b7-b9bb-3f3ec143a87b}`,
/// Only do this if you think it's causing issues, and prefer to check for it's existence using
/// [`did_vmware_bork_my_machine`] first.
fn nuke_3d09c1ca() -> std::io::Result<()> {
    use winreg::{enums::*, reg_key::RegKey};
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.delete_subkey(r#"SOFTWARE\Classes\CLSID{3d09c1ca-2bcc-40b7-b9bb-3f3ec143a87b}"#)
}
