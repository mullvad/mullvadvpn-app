use crate::{
    config::{Config, ConfigFile, VmConfig, VmType},
    package,
};
use anyhow::{Context, Result};
use std::net::IpAddr;

mod logging;
pub mod network;
mod provision;
mod qemu;
mod ssh;
#[cfg(target_os = "macos")]
mod tart;
mod update;
mod util;

#[async_trait::async_trait]
pub trait VmInstance {
    /// Path to pty on the host that corresponds to the serial device
    fn get_pty(&self) -> &str;

    /// Get initial IP address of guest
    fn get_ip(&self) -> &IpAddr;

    /// Wait for VM to destruct
    async fn wait(&mut self);
}

pub async fn set_config(config: &mut ConfigFile, vm_name: &str, vm_config: VmConfig) -> Result<()> {
    config
        .edit(|config| {
            config.vms.insert(vm_name.to_owned(), vm_config);
        })
        .await
        .context("Failed to update VM config")
}

pub async fn run(config: &Config, name: &str) -> Result<Box<dyn VmInstance>> {
    let vm_conf = get_vm_config(config, name)?;

    log::info!("Starting \"{name}\"");

    let instance = match vm_conf.vm_type {
        VmType::Qemu => Box::new(
            qemu::run(config, vm_conf)
                .await
                .context("Failed to run QEMU VM")?,
        ) as Box<_>,
        #[cfg(target_os = "macos")]
        VmType::Tart => Box::new(
            tart::run(config, vm_conf)
                .await
                .context("Failed to run Tart VM")?,
        ) as Box<_>,
        #[cfg(not(target_os = "macos"))]
        VmType::Tart => return Err(anyhow::anyhow!("Failed to run Tart VM on a non-macOS host")),
    };

    log::info!("Started instance of \"{name}\" vm");

    Ok(instance)
}

pub async fn provision(
    config: &Config,
    name: &str,
    instance: &dyn VmInstance,
    app_manifest: &package::Manifest,
) -> Result<String> {
    let vm_config = get_vm_config(config, name)?;
    provision::provision(vm_config, instance, app_manifest).await
}

pub async fn update_packages(
    config: VmConfig,
    instance: &dyn VmInstance,
) -> Result<crate::vm::update::Update> {
    let guest_ip = *instance.get_ip();
    tokio::task::spawn_blocking(move || update::packages(&config, guest_ip)).await?
}

pub fn get_vm_config<'a>(config: &'a Config, name: &str) -> Result<&'a VmConfig> {
    config
        .get_vm(name)
        .with_context(|| format!("Could not find config: {name}"))
}
