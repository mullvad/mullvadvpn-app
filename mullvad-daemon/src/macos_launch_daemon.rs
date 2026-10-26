//! Register Mullvad VPN as a launch daemon/system service on macOS.

use std::io;

use log::trace;
use smappservice_rs::{AppService, ServiceStatus, ServiceType};

pub fn register() -> io::Result<()> {
    match app().register() {
        Ok(()) => {
            trace!("Successfully registered Mullvad VPN LaunchDaemon");
            Ok(())
        }
        Err(err) => Err(io::Error::from_raw_os_error(err.code() as i32)),
    }
}

pub fn status() -> ServiceStatus {
    app().status()
}

fn app() -> AppService {
    AppService::new(ServiceType::Daemon {
        plist_name: "net.mullvad.daemon.plist",
    })
}
