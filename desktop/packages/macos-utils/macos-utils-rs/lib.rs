//! Register Mullvad VPN as a launch daemon/system service on macOS.
#![cfg(target_os = "macos")]

use neon::prelude::{Context, FunctionContext, JsResult, ModuleContext, NeonResult};
use neon::types::{JsValue, Value};
use smappservice_rs::{AppService, ServiceType};

#[neon::main]
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
    cx.export_function("checkLaunchDaemonStatus", check_launch_daemon_status)?;
    Ok(())
}

/// Check Mullvad VPN LaunchDaemon status. See SmAppService for more information.
fn check_launch_daemon_status(mut cx: FunctionContext<'_>) -> JsResult<'_, JsValue> {
    let status = app().status();
    Ok(cx.number(status as isize as f64).as_value(&mut cx))
}

fn app() -> AppService {
    AppService::new(ServiceType::Daemon {
        plist_name: "net.mullvad.daemon.plist",
    })
}
