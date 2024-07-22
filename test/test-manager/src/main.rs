mod config;
mod container;
mod logging;
mod mullvad_daemon;
mod network_monitor;
mod package;
mod run_tests;
mod summary;
mod tests;
mod vm;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use tests::config::DEFAULT_MULLVAD_HOST;

/// Test manager for Mullvad VPN app
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Create or edit a VM config
    Set {
        /// Name of the config
        name: String,

        /// VM config
        #[clap(flatten)]
        config: config::VmConfig,
    },

    /// Remove specified VM config
    Remove {
        /// Name of the config
        name: String,
    },

    /// List available VM configurations
    List,

    /// Spawn a runner instance without running any tests
    RunVm {
        /// Name of the VM config
        vm: String,

        /// Run VNC server on a specified port
        #[arg(long)]
        vnc: Option<u16>,

        /// Make permanent changes to image
        #[arg(long)]
        keep_changes: bool,
    },

    /// List all tests and their priority.
    ListTests,

    /// Spawn a runner instance and run tests
    RunTests {
        /// Name of the VM config
        #[arg(long)]
        vm: String,

        /// Show display of guest
        #[arg(long, group = "display_args")]
        display: bool,

        /// Run VNC server on a specified port
        #[arg(long, group = "display_args")]
        vnc: Option<u16>,

        /// Account number to use for testing
        #[arg(long, short)]
        account: String,

        /// App package to test. Can be a path to the package, just the package file name, git hash
        /// or tag. If the direct path is not given, the package is assumed to be in the directory
        /// specified by the `--package-folder` argument.
        ///
        /// # Note
        ///
        /// The gRPC interface must be compatible with the version specified for
        /// `mullvad-management-interface` in Cargo.toml.
        #[arg(long)]
        app_package: String,

        /// App package to upgrade from when running `test_install_previous_app`, can be left empty
        /// if this test is not ran. Parsed the same way as `--app-package`.
        ///
        /// # Note
        ///
        /// The CLI interface must be compatible with the upgrade test.
        #[arg(long)]
        app_package_to_upgrade_from: Option<String>,

        /// Folder to search for packages. Defaults to current directory.
        #[arg(long, value_name = "DIR")]
        package_folder: Option<PathBuf>,

        /// Only run tests matching substrings
        test_filters: Vec<String>,

        /// Print results live
        #[arg(long, short)]
        verbose: bool,

        /// Output test results in a structured format.
        #[arg(long)]
        test_report: Option<PathBuf>,
    },

    /// Output an HTML-formatted summary of one or more reports
    FormatTestReports {
        /// One or more test reports output by 'test-manager run-tests --test-report'
        reports: Vec<PathBuf>,
    },

    /// Update the system image
    ///
    /// Note that in order for the updates to take place, the VM's config need
    /// to have `provisioner` set to `ssh`, `ssh_user` & `ssh_password` set and
    /// the `ssh_user` should be able to execute commands with sudo/ as root.
    Update {
        /// Name of the VM config
        name: String,
    },
}

#[cfg(target_os = "linux")]
impl Args {
    fn get_vnc_port(&self) -> Option<u16> {
        match self.cmd {
            Commands::RunTests { vnc, .. } | Commands::RunVm { vnc, .. } => vnc,
            _ => None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::Logger::get_or_init();

    let args = Args::parse();

    #[cfg(target_os = "linux")]
    container::relaunch_with_rootlesskit(args.get_vnc_port()).await;

    let config_path = dirs::config_dir()
        .context("Config directory not found. Can not load VM config")?
        .join("mullvad-test")
        .join("config.json");

    let mut config = config::ConfigFile::load_or_default(config_path)
        .await
        .context("Failed to load config")?;
    match args.cmd {
        Commands::Set {
            name,
            config: vm_config,
        } => vm::set_config(&mut config, &name, vm_config)
            .await
            .context("Failed to edit or create VM config"),
        Commands::Remove { name } => {
            if config.get_vm(&name).is_none() {
                println!("No such configuration");
                return Ok(());
            }
            config
                .edit(|config| {
                    config.vms.remove_entry(&name);
                })
                .await
                .context("Failed to remove config entry")?;
            println!("Removed configuration \"{name}\"");
            Ok(())
        }
        Commands::List => {
            println!("Available configurations:");
            for (name, config) in config.vms.iter() {
                println!("{name}: {config:#?}",);
            }
            Ok(())
        }
        Commands::RunVm {
            vm,
            vnc,
            keep_changes,
        } => {
            let mut config = config.clone();
            config.runtime_opts.keep_changes = keep_changes;
            config.runtime_opts.display = if vnc.is_some() {
                config::Display::Vnc
            } else {
                config::Display::Local
            };

            let mut instance = vm::run(&config, &vm).await.context("Failed to start VM")?;

            instance.wait().await;

            Ok(())
        }
        Commands::ListTests => {
            println!("priority\tname");
            for test in tests::get_tests() {
                println!(
                    "{priority:8}\t{name}",
                    name = test.name,
                    priority = test.priority.unwrap_or(0),
                );
            }
            Ok(())
        }
        Commands::RunTests {
            vm,
            display,
            vnc,
            account,
            app_package,
            app_package_to_upgrade_from,
            package_folder,
            test_filters,
            verbose,
            test_report,
        } => {
            let mut config = config.clone();
            config.runtime_opts.display = match (display, vnc.is_some()) {
                (false, false) => config::Display::None,
                (true, false) => config::Display::Local,
                (false, true) => config::Display::Vnc,
                (true, true) => unreachable!("invalid combination"),
            };

            let mullvad_host = config
                .mullvad_host
                .clone()
                .unwrap_or(DEFAULT_MULLVAD_HOST.to_owned());
            log::debug!("Mullvad host: {mullvad_host}");

            let vm_config = vm::get_vm_config(&config, &vm).context("Cannot get VM config")?;

            let summary_logger = match test_report {
                Some(path) => Some(
                    summary::SummaryLogger::new(
                        &vm,
                        test_rpc::meta::Os::from(vm_config.os_type),
                        &path,
                    )
                    .await
                    .context("Failed to create summary logger")?,
                ),
                None => None,
            };

            let manifest = package::get_app_manifest(
                vm_config,
                app_package,
                app_package_to_upgrade_from,
                package_folder,
            )
            .context("Could not find the specified app packages")?;

            let mut instance = vm::run(&config, &vm).await.context("Failed to start VM")?;
            let artifacts_dir = vm::provision(&config, &vm, &*instance, &manifest)
                .await
                .context("Failed to run provisioning for VM")?;

            // For convenience, spawn a SOCKS5 server that is reachable for tests that need it
            let socks = socks_server::spawn(SocketAddr::new(
                crate::vm::network::NON_TUN_GATEWAY.into(),
                crate::vm::network::SOCKS5_PORT,
            ))
            .await?;

            let skip_wait = vm_config.provisioner != config::Provisioner::Noop;

            let result = run_tests::run(
                tests::config::TestConfig {
                    account_number: account,
                    artifacts_dir,
                    app_package_filename: manifest
                        .app_package_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    app_package_to_upgrade_from_filename: manifest
                        .app_package_to_upgrade_from_path
                        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned()),
                    ui_e2e_tests_filename: manifest
                        .ui_e2e_tests_path
                        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned()),
                    mullvad_host,
                    #[cfg(target_os = "macos")]
                    host_bridge_name: crate::vm::network::macos::find_vm_bridge()?,
                    #[cfg(not(target_os = "macos"))]
                    host_bridge_name: crate::vm::network::linux::BRIDGE_NAME.to_owned(),
                    os: test_rpc::meta::Os::from(vm_config.os_type),
                },
                &*instance,
                &test_filters,
                skip_wait,
                !verbose,
                summary_logger,
            )
            .await
            .context("Tests failed");

            if display {
                instance.wait().await;
            }
            socks.close();
            result
        }
        Commands::FormatTestReports { reports } => {
            summary::print_summary_table(&reports).await;
            Ok(())
        }
        Commands::Update { name } => {
            let vm_config = vm::get_vm_config(&config, &name).context("Cannot get VM config")?;

            let instance = vm::run(&config, &name)
                .await
                .context("Failed to start VM")?;

            let update_output = vm::update_packages(vm_config.clone(), &*instance)
                .await
                .context("Failed to update packages to the VM image")?;
            log::info!("Update command finished with output: {}", &update_output);
            // TODO: If the update was successful, commit the changes to the VM image.
            log::info!("Note: updates have not been persisted to the image");
            Ok(())
        }
    }
}
