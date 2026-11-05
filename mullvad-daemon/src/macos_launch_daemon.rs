//! Register Mullvad VPN as a launch daemon/system service on macOS.

use std::io;

use log::trace;
use smappservice_rs::{AppService, ServiceStatus, ServiceType};

pub fn status() -> ServiceStatus {
    app().status()
}

fn app() -> AppService {
    AppService::new(ServiceType::Daemon {
        plist_name: "net.mullvad.daemon.plist",
    })
}
