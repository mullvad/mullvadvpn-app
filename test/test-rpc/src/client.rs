use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, SystemTime},
};

use crate::mullvad_daemon::ServiceStatus;

use super::*;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(300);
const REBOOT_TIMEOUT: Duration = Duration::from_secs(30);
const LOG_LEVEL_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct ServiceClient {
    connection_handle: transport::ConnectionHandle,
    client: service::ServiceClient,
}

impl ServiceClient {
    pub fn new(
        connection_handle: transport::ConnectionHandle,
        transport: tarpc::transport::channel::UnboundedChannel<
            tarpc::Response<service::ServiceResponse>,
            tarpc::ClientMessage<service::ServiceRequest>,
        >,
    ) -> Self {
        Self {
            connection_handle,
            client: super::service::ServiceClient::new(tarpc::client::Config::default(), transport)
                .spawn(),
        }
    }

    /// Install app package.
    pub async fn install_app(&self, package_path: package::Package) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(INSTALL_TIMEOUT).unwrap();

        self.client
            .install_app(ctx, package_path)
            .await
            .map_err(Error::Tarpc)?
    }

    /// Remove app package.
    pub async fn uninstall_app(&self, env: HashMap<String, String>) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(INSTALL_TIMEOUT).unwrap();

        self.client.uninstall_app(ctx, env).await?
    }

    /// Execute a program with additional environment-variables set.
    pub async fn exec_env<
        I: IntoIterator<Item = T>,
        M: IntoIterator<Item = (K, T)>,
        T: AsRef<str>,
        K: AsRef<str>,
    >(
        &self,
        path: T,
        args: I,
        env: M,
    ) -> Result<ExecResult, Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(INSTALL_TIMEOUT).unwrap();
        self.client
            .exec(
                ctx,
                path.as_ref().to_string(),
                args.into_iter().map(|v| v.as_ref().to_string()).collect(),
                env.into_iter()
                    .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
                    .collect(),
            )
            .await?
    }

    /// Execute a program.
    pub async fn exec<I: IntoIterator<Item = T>, T: AsRef<str>>(
        &self,
        path: T,
        args: I,
    ) -> Result<ExecResult, Error> {
        let env: [(&str, T); 0] = [];
        self.exec_env(path, args, env).await
    }

    /// Get the output of the runners stdout logs since the last time this function was called.
    /// Block if there is no output until some output is provided by the runner.
    pub async fn poll_output(&self) -> Result<Vec<logging::Output>, Error> {
        self.client.poll_output(tarpc::context::current()).await?
    }

    /// Get the output of the runners stdout logs since the last time this function was called.
    /// Block if there is no output until some output is provided by the runner.
    pub async fn try_poll_output(&self) -> Result<Vec<logging::Output>, Error> {
        self.client
            .try_poll_output(tarpc::context::current())
            .await?
    }

    pub async fn get_mullvad_app_logs(&self) -> Result<logging::LogOutput, Error> {
        self.client
            .get_mullvad_app_logs(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)
    }

    /// Return the OS of the guest.
    pub async fn get_os(&self) -> Result<meta::Os, Error> {
        self.client
            .get_os(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)
    }

    /// Wait for the Mullvad service to enter a specified state. The state is inferred from the presence
    /// of a named pipe or UDS, not the actual system service state.
    pub async fn mullvad_daemon_wait_for_state(
        &self,
        accept_state_fn: impl Fn(ServiceStatus) -> bool,
    ) -> Result<mullvad_daemon::ServiceStatus, Error> {
        const MAX_ATTEMPTS: usize = 10;
        const POLL_INTERVAL: Duration = Duration::from_secs(3);

        for _ in 0..MAX_ATTEMPTS {
            let last_state = self.mullvad_daemon_get_status().await?;
            match accept_state_fn(last_state) {
                true => return Ok(last_state),
                false => tokio::time::sleep(POLL_INTERVAL).await,
            }
        }
        Err(Error::Timeout)
    }

    /// Return status of the system service. The state is inferred from the presence of
    /// a named pipe or UDS, not the actual system service state.
    pub async fn mullvad_daemon_get_status(&self) -> Result<mullvad_daemon::ServiceStatus, Error> {
        self.client
            .mullvad_daemon_get_status(tarpc::context::current())
            .await
            .map_err(Error::Tarpc)
    }

    /// Returns all Mullvad app files, directories, and other data found on the system.
    pub async fn find_mullvad_app_traces(&self) -> Result<Vec<AppTrace>, Error> {
        self.client
            .find_mullvad_app_traces(tarpc::context::current())
            .await?
    }

    /// Returns path of Mullvad app cache directorie on the test runner.
    pub async fn find_mullvad_app_cache_dir(&self) -> Result<PathBuf, Error> {
        self.client
            .get_mullvad_app_cache_dir(tarpc::context::current())
            .await?
    }

    /// Send TCP packet
    pub async fn send_tcp(
        &self,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), Error> {
        self.client
            .send_tcp(tarpc::context::current(), interface, bind_addr, destination)
            .await?
    }

    /// Send UDP packet
    pub async fn send_udp(
        &self,
        interface: Option<String>,
        bind_addr: SocketAddr,
        destination: SocketAddr,
    ) -> Result<(), Error> {
        self.client
            .send_udp(tarpc::context::current(), interface, bind_addr, destination)
            .await?
    }

    /// Send ICMP
    pub async fn send_ping(
        &self,
        interface: Option<String>,
        destination: IpAddr,
    ) -> Result<(), Error> {
        self.client
            .send_ping(tarpc::context::current(), interface, destination)
            .await?
    }

    /// Fetch the current location.
    pub async fn geoip_lookup(&self, mullvad_host: String) -> Result<AmIMullvad, Error> {
        self.client
            .geoip_lookup(tarpc::context::current(), mullvad_host)
            .await?
    }

    /// Returns the IP of the given interface.
    pub async fn get_interface_ip(&self, interface: String) -> Result<IpAddr, Error> {
        self.client
            .get_interface_ip(tarpc::context::current(), interface)
            .await?
    }

    /// Returns the name of the default non-tunnel interface
    pub async fn get_default_interface(&self) -> Result<String, Error> {
        self.client
            .get_default_interface(tarpc::context::current())
            .await?
    }

    pub async fn resolve_hostname(&self, hostname: String) -> Result<Vec<SocketAddr>, Error> {
        self.client
            .resolve_hostname(tarpc::context::current(), hostname)
            .await?
    }

    pub async fn restart_app(&self) -> Result<(), Error> {
        let _ = self.client.restart_app(tarpc::context::current()).await?;
        Ok(())
    }

    /// Stop the app.
    ///
    /// Shuts down a running app, making it disconnect from any current tunnel
    /// connection and making it write to caches.
    ///
    /// # Note
    /// This function will return *after* the app has been stopped, thus
    /// blocking execution until then.
    pub async fn stop_app(&self) -> Result<(), Error> {
        let _ = self.client.stop_app(tarpc::context::current()).await?;
        Ok(())
    }

    /// Start the app.
    ///
    /// # Note
    /// This function will return *after* the app has been start, thus
    /// blocking execution until then.
    pub async fn start_app(&self) -> Result<(), Error> {
        let _ = self.client.start_app(tarpc::context::current()).await?;
        Ok(())
    }

    pub async fn set_daemon_log_level(
        &self,
        verbosity_level: mullvad_daemon::Verbosity,
    ) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(LOG_LEVEL_TIMEOUT).unwrap();
        self.client
            .set_daemon_log_level(ctx, verbosity_level)
            .await??;

        self.mullvad_daemon_wait_for_state(|state| state == ServiceStatus::Running)
            .await?;

        Ok(())
    }

    pub async fn set_daemon_environment(&self, env: HashMap<String, String>) -> Result<(), Error> {
        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(LOG_LEVEL_TIMEOUT).unwrap();
        self.client.set_daemon_environment(ctx, env).await??;

        self.mullvad_daemon_wait_for_state(|state| state == ServiceStatus::Running)
            .await?;

        Ok(())
    }

    pub async fn copy_file(&self, src: String, dest: String) -> Result<(), Error> {
        log::debug!("Copying \"{src}\" to \"{dest}\"");
        self.client
            .copy_file(tarpc::context::current(), src, dest)
            .await?
    }

    pub async fn write_file(&self, dest: impl AsRef<Path>, bytes: Vec<u8>) -> Result<(), Error> {
        log::debug!(
            "Writing {bytes} bytes to \"{file}\"",
            bytes = bytes.len(),
            file = dest.as_ref().display()
        );
        self.client
            .write_file(
                tarpc::context::current(),
                dest.as_ref().to_path_buf(),
                bytes,
            )
            .await?
    }

    pub async fn reboot(&mut self) -> Result<(), Error> {
        log::debug!("Rebooting server");

        let mut ctx = tarpc::context::current();
        ctx.deadline = SystemTime::now().checked_add(REBOOT_TIMEOUT).unwrap();

        self.client.reboot(ctx).await??;
        self.connection_handle.reset_connected_state().await;
        self.connection_handle.wait_for_server().await?;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        Ok(())
    }

    pub async fn set_mullvad_daemon_service_state(&self, on: bool) -> Result<(), Error> {
        self.client
            .set_mullvad_daemon_service_state(tarpc::context::current(), on)
            .await??;

        self.mullvad_daemon_wait_for_state(|state| {
            if on {
                state == ServiceStatus::Running
            } else {
                state == ServiceStatus::NotRunning
            }
        })
        .await?;

        Ok(())
    }

    pub async fn make_device_json_old(&self) -> Result<(), Error> {
        self.client
            .make_device_json_old(tarpc::context::current())
            .await?
    }
}
